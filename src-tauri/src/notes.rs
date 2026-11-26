use crate::models::{Note, NoteReminder, ReminderInput, ReminderKind, ReminderStatus};
use chrono::{Duration, Local, NaiveDateTime};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Zero-width space used to escape body lines that look like note headers.
const ZWSP: char = '\u{200B}';

/// Prefix for the reminder metadata HTML comment stored under each note header.
const REMINDER_PREFIX: &str = "<!--qn-reminder:";
const REMINDER_SUFFIX: &str = "-->";

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

/// Split optional reminder comment from the raw body block.
fn extract_reminder(raw_body: &str) -> (Option<NoteReminder>, String) {
    let trimmed_start = raw_body.trim_start_matches('\n');
    let mut lines = trimmed_start.lines();
    let Some(first) = lines.next() else {
        return (None, String::new());
    };
    let first_trim = first.trim();
    if let Some(json) = first_trim
        .strip_prefix(REMINDER_PREFIX)
        .and_then(|s| s.strip_suffix(REMINDER_SUFFIX))
    {
        let reminder = serde_json::from_str::<NoteReminder>(json.trim()).ok();
        let rest: String = lines.collect::<Vec<_>>().join("\n");
        // Preserve a leading blank line after the comment as body start.
        let body = if rest.starts_with('\n') {
            rest
        } else if rest.is_empty() {
            String::new()
        } else {
            rest
        };
        return (reminder, body);
    }
    (None, trimmed_start.to_string())
}

fn format_reminder_line(reminder: &NoteReminder) -> String {
    let json = serde_json::to_string(reminder).unwrap_or_else(|_| "{}".into());
    format!("{REMINDER_PREFIX}{json}{REMINDER_SUFFIX}")
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
            let body_text = if let Some(rest) = trimmed.strip_prefix('\n') {
                rest
            } else {
                trimmed
            };
            let (reminder, body_without_meta) = extract_reminder(body_text);
            let body_text = unescape_body(&body_without_meta);
            let id = format!("{project_id}:{index}");
            notes.push(Note {
                id,
                project_id: project_id.to_string(),
                timestamp: timestamp.clone(),
                body: body_text,
                reminder,
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
        if let Some(reminder) = &note.reminder {
            out.push_str(&format_reminder_line(reminder));
            out.push('\n');
        }
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

/// Normalize UI datetime strings to `YYYY-MM-DD HH:mm`.
pub fn normalize_local_datetime(raw: &str) -> Result<String, String> {
    let s = raw.trim().replace('T', " ");
    let formats = ["%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"];
    for fmt in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&s, fmt) {
            return Ok(dt.format("%Y-%m-%d %H:%M").to_string());
        }
    }
    // date-only → 09:00
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d") {
        return Ok(format!("{} 09:00", d.format("%Y-%m-%d")));
    }
    Err(format!("Invalid datetime: {raw}"))
}

pub fn parse_local_datetime(s: &str) -> Result<NaiveDateTime, String> {
    let normalized = normalize_local_datetime(s)?;
    NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M")
        .map_err(|e| format!("Invalid datetime {s}: {e}"))
}

pub fn format_now_local() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

pub fn build_reminder_from_input(input: &ReminderInput) -> Result<NoteReminder, String> {
    match input.kind {
        ReminderKind::Relative => {
            let days = input.relative_days.unwrap_or(0);
            if days == 0 {
                return Err("Укажите число дней (≥ 1)".into());
            }
            let due = Local::now() + Duration::days(days as i64);
            let due_at = due.format("%Y-%m-%d %H:%M").to_string();
            Ok(NoteReminder {
                kind: ReminderKind::Relative,
                due_at,
                relative_days: Some(days),
                status: ReminderStatus::Pending,
                snooze_until: None,
                notified_at: None,
            })
        }
        ReminderKind::Absolute => {
            let raw = input
                .absolute_at
                .as_deref()
                .ok_or_else(|| "Укажите дату и время напоминания".to_string())?;
            let due_at = normalize_local_datetime(raw)?;
            Ok(NoteReminder {
                kind: ReminderKind::Absolute,
                due_at,
                relative_days: None,
                status: ReminderStatus::Pending,
                snooze_until: None,
                notified_at: None,
            })
        }
    }
}

pub fn snooze_until_from_minutes(minutes: u32) -> String {
    let until = Local::now() + Duration::minutes(minutes as i64);
    until.format("%Y-%m-%d %H:%M").to_string()
}

/// Tomorrow at 09:00 local.
pub fn snooze_until_tomorrow_morning() -> String {
    let now = Local::now();
    let tomorrow = (now + Duration::days(1)).date_naive();
    let dt = tomorrow
        .and_hms_opt(9, 0, 0)
        .expect("09:00 is a valid time");
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub fn effective_due(reminder: &NoteReminder) -> &str {
    match reminder.status {
        ReminderStatus::Snoozed => reminder
            .snooze_until
            .as_deref()
            .unwrap_or(reminder.due_at.as_str()),
        _ => reminder.due_at.as_str(),
    }
}

pub fn append_note(
    storage_dir: &str,
    folder_name: &str,
    project_id: &str,
    text: &str,
    reminder: Option<NoteReminder>,
) -> Result<Note, String> {
    let mut notes = read_project_notes(storage_dir, folder_name, project_id)?;
    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let index = notes.len();
    let note = Note {
        id: format!("{project_id}:{index}"),
        project_id: project_id.to_string(),
        timestamp,
        body: text.trim_end().to_string(),
        reminder,
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
        assert!(notes[0].reminder.is_none());
    }

    #[test]
    fn parses_multiple_notes() {
        let content = "## 2026-11-26 05:52\none\n\n## 2026-11-26 06:00\ntwo\n";
        let notes = parse_notes("p1", content);
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].body, "one");
        assert_eq!(notes[1].body, "two");
    }

    #[test]
    fn roundtrips_reminder_metadata() {
        let reminder = NoteReminder {
            kind: ReminderKind::Relative,
            due_at: "2026-11-28 14:30".into(),
            relative_days: Some(2),
            status: ReminderStatus::Pending,
            snooze_until: None,
            notified_at: None,
        };
        let notes = vec![Note {
            id: "p1:0".into(),
            project_id: "p1".into(),
            timestamp: "2026-11-26 14:30".into(),
            body: "call mom".into(),
            reminder: Some(reminder.clone()),
        }];
        let content = serialize_notes(&notes);
        assert!(content.contains("<!--qn-reminder:"));
        let parsed = parse_notes("p1", &content);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].body, "call mom");
        assert_eq!(parsed[0].reminder.as_ref(), Some(&reminder));
    }

    #[test]
    fn legacy_notes_without_reminder_still_parse() {
        let content = "## 2026-11-26 05:52\nplain note\n";
        let notes = parse_notes("p1", content);
        assert_eq!(notes.len(), 1);
        assert!(notes[0].reminder.is_none());
        assert_eq!(notes[0].body, "plain note");
    }

    #[test]
    fn builds_absolute_reminder() {
        let input = ReminderInput {
            kind: ReminderKind::Absolute,
            relative_days: None,
            absolute_at: Some("2026-12-01T18:00".into()),
        };
        let r = build_reminder_from_input(&input).unwrap();
        assert_eq!(r.due_at, "2026-12-01 18:00");
        assert_eq!(r.kind, ReminderKind::Absolute);
    }
}
