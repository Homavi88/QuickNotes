//! Custom app-data location via a small pointer file.
//!
//! Layout when relocated: `{chosenParent}/QuickNotes/data/` (config + notes + logs).
//! Pointer (`data-root.txt`) next to the exe and/or under fixed AppData bootstrap
//! so the next launch finds the new folder without moving the portable exe.

use crate::models::{config_dir, exe_dir, is_portable_mode, AppConfig};
use crate::startup;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const POINTER_FILE: &str = "data-root.txt";

/// Absolute parent chosen by the user (QuickNotes/data is created under it).
pub fn read_data_parent_override() -> Option<PathBuf> {
    for pointer in pointer_candidates() {
        if let Some(parent) = read_pointer(&pointer) {
            return Some(parent);
        }
    }
    None
}

pub fn data_dir_for_parent(parent: &Path) -> PathBuf {
    parent.join("QuickNotes").join("data")
}

/// Default parent shown in Settings (not used when an override pointer exists).
pub fn default_data_parent() -> PathBuf {
    if is_portable_mode() {
        return exe_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// Current chosen parent if relocated; otherwise `None` (using built-in default layout).
pub fn current_data_parent() -> Option<PathBuf> {
    read_data_parent_override()
}

fn pointer_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(dir) = exe_dir() {
        v.push(dir.join(POINTER_FILE));
    }
    // Fixed AppData bootstrap — survives relocate when exe dir is not writable (MSI).
    if let Some(dir) = dirs::config_dir() {
        v.push(dir.join("QuickNotes").join(POINTER_FILE));
    }
    v
}

fn read_pointer(path: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(path).ok()?;
    let line = raw.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    let parent = PathBuf::from(line);
    if parent.as_os_str().is_empty() {
        return None;
    }
    Some(parent)
}

fn write_pointer(path: &Path, parent: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create pointer dir: {e}"))?;
    }
    let tmp = path.with_extension("txt.tmp");
    {
        let mut f =
            File::create(&tmp).map_err(|e| format!("Failed to write pointer temp: {e}"))?;
        writeln!(f, "{}", parent.display()).map_err(|e| format!("Failed to write pointer: {e}"))?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("Failed to finalize pointer: {e}"))?;
    Ok(())
}

/// Write pointer next to exe (preferred) and AppData bootstrap (fallback / MSI).
pub fn write_data_parent_pointers(parent: &Path) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();
    let mut last_err: Option<String> = None;
    for path in pointer_candidates() {
        match write_pointer(&path, parent) {
            Ok(()) => {
                startup::log_line(&format!(
                    "data-root pointer written: {} -> {}",
                    path.display(),
                    parent.display()
                ));
                written.push(path);
            }
            Err(e) => {
                startup::log_line(&format!(
                    "data-root pointer write failed ({}): {e}",
                    path.display()
                ));
                last_err = Some(e);
            }
        }
    }
    if written.is_empty() {
        return Err(last_err.unwrap_or_else(|| "Could not write data-root pointer".into()));
    }
    Ok(written)
}

pub fn clear_data_parent_pointers() -> Result<(), String> {
    for path in pointer_candidates() {
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
            startup::log_line(&format!("data-root pointer removed: {}", path.display()));
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create {}: {e}", dst.display()))?;
    let entries = fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let ft = entry
            .file_type()
            .map_err(|e| format!("Failed to stat {}: {e}", from.display()))?;
        if ft.is_dir() {
            // Skip copying into ourselves if nested oddly.
            if from == dst {
                continue;
            }
            copy_dir_recursive(&from, &to)?;
        } else if ft.is_file() {
            // Never copy pointer/MOVED markers as data payload.
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == POINTER_FILE || name == "MOVED.txt" {
                continue;
            }
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
            }
            fs::copy(&from, &to)
                .map_err(|e| format!("Failed to copy {} → {}: {e}", from.display(), to.display()))?;
        }
    }
    Ok(())
}

fn dir_has_payload(dir: &Path) -> bool {
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    rd.flatten().any(|e| {
        let n = e.file_name();
        let n = n.to_string_lossy();
        n != POINTER_FILE && n != "MOVED.txt"
    })
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    let na = fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let nb = fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    na == nb
}

/// Relocate app data to `{parent}/QuickNotes/data`. Optionally copy from the current data dir.
/// Does not delete the old location. Returns the new config dir.
pub fn relocate_data_parent(
    parent: &Path,
    copy_existing: bool,
) -> Result<(PathBuf, AppConfig), String> {
    let parent = parent
        .canonicalize()
        .unwrap_or_else(|_| parent.to_path_buf());
    if parent.as_os_str().is_empty() {
        return Err("Data parent path is empty".into());
    }

    let old_dir = config_dir();
    let new_dir = data_dir_for_parent(&parent);

    if paths_equal(&old_dir, &new_dir) {
        // Already there — refresh pointers and return current config.
        let _ = write_data_parent_pointers(&parent);
        let cfg = crate::config::load_config()?;
        return Ok((new_dir, cfg));
    }

    fs::create_dir_all(&new_dir)
        .map_err(|e| format!("Failed to create {}: {e}", new_dir.display()))?;

    if copy_existing && old_dir.exists() && dir_has_payload(&old_dir) {
        startup::log_line(&format!(
            "Copying data {} → {}",
            old_dir.display(),
            new_dir.display()
        ));
        copy_dir_recursive(&old_dir, &new_dir)?;
    }

    write_data_parent_pointers(&parent)?;

    // Bootstrap marker inside the new data folder (helps humans / support).
    let marker = new_dir.join(".quicknotes-data");
    let _ = fs::write(
        &marker,
        format!(
            "QuickNotes data folder\nparent={}\n",
            parent.display()
        ),
    );

    if old_dir.exists() && !paths_equal(&old_dir, &new_dir) {
        let notice = old_dir.join("MOVED.txt");
        let _ = fs::write(
            &notice,
            format!(
                "QuickNotes data was relocated.\nNew location:\n{}\n\nThis folder was not deleted.\n",
                new_dir.display()
            ),
        );
    }

    // Load config from the new location (pointers already active).
    let mut cfg = crate::config::load_config()?;

    // If notes storage still pointed under the old data tree, retarget it.
    let old_prefix = old_dir.to_string_lossy();
    let storage = cfg.settings.storage_dir.clone();
    if storage.starts_with(old_prefix.as_ref()) {
        let rest = storage[old_prefix.len()..]
            .trim_start_matches(['/', '\\']);
        let retargeted = if rest.is_empty() {
            new_dir.join("notes")
        } else {
            new_dir.join(rest)
        };
        cfg.settings.storage_dir = retargeted.to_string_lossy().into_owned();
        crate::config::save_config(&cfg)?;
    } else if cfg.settings.storage_dir.trim().is_empty() {
        cfg.settings.storage_dir = crate::models::default_storage_dir();
        crate::config::save_config(&cfg)?;
    }

    let _ = fs::create_dir_all(&cfg.settings.storage_dir);

    startup::log_line(&format!(
        "Data relocated to {} (copy_existing={copy_existing})",
        new_dir.display()
    ));

    Ok((new_dir, cfg))
}

/// Clear override pointers and return to default layout (exe/data or AppData). Does not delete relocated data.
pub fn reset_data_parent_to_default(copy_back: bool) -> Result<(PathBuf, AppConfig), String> {
    let old_dir = config_dir();
    clear_data_parent_pointers()?;
    let new_dir = config_dir(); // now default

    if copy_back && old_dir.exists() && !paths_equal(&old_dir, &new_dir) && dir_has_payload(&old_dir)
    {
        fs::create_dir_all(&new_dir)
            .map_err(|e| format!("Failed to create {}: {e}", new_dir.display()))?;
        copy_dir_recursive(&old_dir, &new_dir)?;
    }

    let mut cfg = crate::config::load_config()?;
    let old_prefix = old_dir.to_string_lossy();
    let storage = cfg.settings.storage_dir.clone();
    if storage.starts_with(old_prefix.as_ref()) {
        let rest = storage[old_prefix.len()..]
            .trim_start_matches(['/', '\\']);
        let retargeted = if rest.is_empty() {
            new_dir.join("notes")
        } else {
            new_dir.join(rest)
        };
        cfg.settings.storage_dir = retargeted.to_string_lossy().into_owned();
        crate::config::save_config(&cfg)?;
    }

    Ok((new_dir, cfg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_for_parent_appends_quicknotes_data() {
        let p = PathBuf::from("/tmp/chosen");
        assert_eq!(
            data_dir_for_parent(&p),
            PathBuf::from("/tmp/chosen/QuickNotes/data")
        );
    }
}
