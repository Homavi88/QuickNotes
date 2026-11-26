use crate::models::Note;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Zero-width space used to escape body lines that look like note headers.
const ZWSP: char = '\u{200B}';

fn header_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^## (\d{4}-\d{2}-\d{2} \d{2}:\d{2})\s*$").unwrap())
}

fn looks_like_header(line: &str) -> bool {
    header_re().is_match(line)
}

/// Escape body lines that would be mistaken for note headers on read.
pub fn escape_body(text: &str) -> String {
    text.lines()
        .map(|line| {
            if looks_like_header(line) {
                format!("{ZWSP}{line}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reverse header escaping when reading a note body.
pub fn unescape_body(text: &str) -> String {
    text.lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix(ZWSP) {
                if looks_like_header(rest) {
                    return rest.to_string();
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn project_notes_path(storage_dir: &str, folder_name: &str) -> PathBuf {
    Path::new(storage_dir).join(folder_name).join("notes.md")
}

pub fn ensure_project_dir(storage_dir: &str, folder_name: &str) -> Result<PathBuf, String> {
    let dir = Path::new(storage_dir).join(folder_name);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create project folder: {e}"))?;
    let notes = dir.join("notes.md");
    if !notes.exists() {
        fs::write(&notes, "").map_err(|e| format!("Failed to create notes.md: {e}"))?;
    }
    Ok(notes)
}

pub fn parse_notes(project_id: &str, content: &str) -> Vec<Note> {
    let mut notes = Vec::new();
    let mut current_ts: Option<String> = None;
    let mut current_body: Vec<String> = Vec::new();
    let mut index = 0usize;

    let flush = |ts: &Option<String>,
                 body: &mut Vec<String>,
                 notes: &mut Vec<Note>,
                 project_id: &str,
                 index: &mut usize| {
        if let Some(timestamp) = ts {
            let raw = body.join("\n");
            let trimmed = raw.trim_end_matches('\n');
            // Drop a single leading blank line after the header for nicer bodies.
            let body_text = if let Some(rest) = trimmed.strip_prefix('\n') {
                rest
            } else {
                trimmed
            };
            let body_text = unescape_body(body_text);
            let id = format!("{project_id}:{index}");
            notes.push(Note {
                id,
                project_id: project_id.to_string(),
                timestamp: timestamp.clone(),
                body: body_text,
            });
            *index += 1;
            body.clear();
        }
    };

    for line in content.lines() {
        // Escaped headers start with ZWSP and must stay in the body.
        if !line.starts_with(ZWSP) {
            if let Some(caps) = header_re().captures(line) {
                flush(
                    &current_ts,
                    &mut current_body,
                    &mut notes,
                    project_id,
                    &mut index,
                );
                current_ts = Some(caps[1].to_string());
                continue;
            }
        }
        if current_ts.is_some() {
            current_body.push(line.to_string());
        }
    }
    flush(
        &current_ts,
        &mut current_body,
        &mut notes,
        project_id,
        &mut index,
    );
    notes
}

pub fn serialize_notes(notes: &[Note]) -> String {
    let mut out = String::new();
    for note in notes {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("## {}\n", note.timestamp));
        let escaped = escape_body(&note.body);
        out.push_str(&escaped);
        if !escaped.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

pub fn read_project_notes(
    storage_dir: &str,
    folder_name: &str,
    project_id: &str,
) -> Result<Vec<Note>, String> {
    let path = project_notes_path(storage_dir, folder_name);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read notes: {e}"))?;
    Ok(parse_notes(project_id, &content))
}

pub fn write_project_notes(
    storage_dir: &str,
    folder_name: &str,
    notes: &[Note],
) -> Result<(), String> {
    let path = ensure_project_dir(storage_dir, folder_name)?;
    let content = serialize_notes(notes);
    // Atomic-ish write: temp file then rename.
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, content.as_bytes()).map_err(|e| format!("Failed to write notes: {e}"))?;
    fs::rename(&tmp, &path).map_err(|e| format!("Failed to finalize notes: {e}"))?;
    Ok(())
}

pub fn append_note(
    storage_dir: &str,
    folder_name: &str,
    project_id: &str,
    text: &str,
) -> Result<Note, String> {
    let mut notes = read_project_notes(storage_dir, folder_name, project_id)?;
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let index = notes.len();
    let note = Note {
        id: format!("{project_id}:{index}"),
        project_id: project_id.to_string(),
        timestamp,
        body: text.trim_end().to_string(),
    };
    notes.push(note.clone());
    write_project_notes(storage_dir, folder_name, &notes)?;
    Ok(note)
}

pub fn sanitize_folder_name(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    s = s.trim().trim_matches('.').to_string();
    if s.is_empty() {
        s = "Project".into();
    }
    // Windows reserved names
    let upper = s.to_ascii_uppercase();
    if matches!(
        upper.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "LPT1" | "LPT2" | "LPT3"
    ) {
        s = format!("{s}_");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_and_parses_header_like_body() {
        let body = "idea\n## 2026-01-01 12:00\nstill body";
        let escaped = escape_body(body);
        assert!(escaped.contains(&format!("{ZWSP}## 2026-01-01 12:00")));
        let content = format!("## 2026-11-26 05:52\n{escaped}\n");
        let notes = parse_notes("p1", &content);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, body);
    }

    #[test]
    fn parses_multiple_notes() {
        let content = "## 2026-11-26 05:52\none\n\n## 2026-11-26 06:00\ntwo\n";
        let notes = parse_notes("p1", content);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].body, "one");
        assert_eq!(notes[1].body, "two");
    }
}
