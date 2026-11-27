use crate::config::{save_config, unique_folder_name};
use crate::models::*;
use crate::notes::{
    append_note, build_reminder_from_input, ensure_project_dir, project_notes_path,
    read_project_notes, sanitize_folder_name, set_note_closed as apply_note_closed,
    snooze_until_from_minutes, snooze_until_tomorrow_morning, write_project_notes,
};
use crate::reminders::{apply_snooze, find_note_mut, mark_reminder_done, reminder_kind_allowed};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

fn find_project<'a>(cfg: &'a AppConfig, id: &str) -> Result<&'a Project, String> {
    cfg.projects
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| "Project not found".to_string())
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = state.config.lock();
    Ok(cfg.clone())
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock();
    let old_hotkey = cfg.settings.hotkey.clone();
    let storage_changed = cfg.settings.storage_dir != settings.storage_dir;
    cfg.settings = settings;
    save_config(&cfg)?;
    let snapshot = cfg.clone();
    drop(cfg);

    if storage_changed {
        fs::create_dir_all(&snapshot.settings.storage_dir)
            .map_err(|e| format!("Failed to create storage dir: {e}"))?;
    }

    // Re-register hotkey if changed (best-effort; JS also handles this).
    if old_hotkey != snapshot.settings.hotkey {
        let _ = app.emit("hotkey-changed", snapshot.settings.hotkey.clone());
    }

    let _ = app.emit("config-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let cfg = state.config.lock();
    let mut projects = cfg.projects.clone();
    projects.sort_by_key(|p| p.order);
    Ok(projects)
}

#[tauri::command]
pub fn create_project(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    let mut cfg = state.config.lock();
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Project name is required".into());
    }
    let base = sanitize_folder_name(name);
    let folder_name = unique_folder_name(&cfg.settings.storage_dir, &base, None, &cfg.projects);
    ensure_project_dir(&cfg.settings.storage_dir, &folder_name)?;

    let order = cfg.projects.iter().map(|p| p.order).max().unwrap_or(-1) + 1;
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        folder_name,
        color: input.color,
        emoji: input.emoji,
        order,
    };
    cfg.projects.push(project.clone());
    if cfg.settings.last_project_id.is_none() {
        cfg.settings.last_project_id = Some(project.id.clone());
    }
    save_config(&cfg)?;
    let _ = app.emit("config-updated", cfg.clone());
    Ok(project)
}

#[tauri::command]
pub fn update_project(
    app: AppHandle,
    state: State<'_, AppState>,
    input: UpdateProjectInput,
) -> Result<Project, String> {
    let mut cfg = state.config.lock();
    let storage = cfg.settings.storage_dir.clone();
    let idx = cfg
        .projects
        .iter()
        .position(|p| p.id == input.id)
        .ok_or_else(|| "Project not found".to_string())?;

    if let Some(name) = input.name {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("Project name is required".into());
        }
        let old_folder = cfg.projects[idx].folder_name.clone();
        let base = sanitize_folder_name(&name);
        let new_folder = unique_folder_name(&storage, &base, Some(&input.id), &cfg.projects);
        if new_folder != old_folder {
            let old_path = std::path::Path::new(&storage).join(&old_folder);
            let new_path = std::path::Path::new(&storage).join(&new_folder);
            if old_path.exists() {
                fs::rename(&old_path, &new_path)
                    .map_err(|e| format!("Failed to rename project folder: {e}"))?;
            } else {
                ensure_project_dir(&storage, &new_folder)?;
            }
            cfg.projects[idx].folder_name = new_folder;
        }
        cfg.projects[idx].name = name;
    }
    if let Some(color) = input.color {
        cfg.projects[idx].color = color;
    }
    if let Some(emoji) = input.emoji {
        cfg.projects[idx].emoji = emoji;
    }

    let project = cfg.projects[idx].clone();
    save_config(&cfg)?;
    let _ = app.emit("config-updated", cfg.clone());
    Ok(project)
}

#[tauri::command]
pub fn delete_project(
    app: AppHandle,
    state: State<'_, AppState>,
    input: DeleteProjectInput,
) -> Result<(), String> {
    let mut cfg = state.config.lock();
    let idx = cfg
        .projects
        .iter()
        .position(|p| p.id == input.id)
        .ok_or_else(|| "Project not found".to_string())?;
    let project = cfg.projects.remove(idx);
    if input.delete_files {
        let path = std::path::Path::new(&cfg.settings.storage_dir).join(&project.folder_name);
        if path.exists() {
            fs::remove_dir_all(&path)
                .map_err(|e| format!("Failed to delete project folder: {e}"))?;
        }
    }
    if cfg.settings.last_project_id.as_deref() == Some(project.id.as_str()) {
        cfg.settings.last_project_id = cfg.projects.first().map(|p| p.id.clone());
    }
    if let Some(draft) = &cfg.draft {
        if draft.project_id.as_deref() == Some(project.id.as_str()) {
            cfg.draft = None;
        }
    }
    // Re-normalize order
    for (i, p) in cfg.projects.iter_mut().enumerate() {
        p.order = i as i32;
    }
    save_config(&cfg)?;
    let _ = app.emit("config-updated", cfg.clone());
    Ok(())
}

#[tauri::command]
pub fn reorder_projects(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ReorderProjectsInput,
) -> Result<Vec<Project>, String> {
    let mut cfg = state.config.lock();
    let mut next = Vec::new();
    for (order, id) in input.ordered_ids.iter().enumerate() {
        let mut p = cfg
            .projects
            .iter()
            .find(|p| &p.id == id)
            .cloned()
            .ok_or_else(|| format!("Unknown project id: {id}"))?;
        p.order = order as i32;
        next.push(p);
    }
    // Keep any projects missing from the list at the end
    for p in &cfg.projects {
        if !next.iter().any(|n| n.id == p.id) {
            let mut copy = p.clone();
            copy.order = next.len() as i32;
            next.push(copy);
        }
    }
    cfg.projects = next;
    save_config(&cfg)?;
    let projects = cfg.projects.clone();
    let _ = app.emit("config-updated", cfg.clone());
    Ok(projects)
}

#[tauri::command]
pub fn list_notes(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<Note>, String> {
    let cfg = state.config.lock();
    let mut all = Vec::new();
    let projects: Vec<&Project> = match project_id.as_deref() {
        Some(id) => vec![find_project(&cfg, id)?],
        None => cfg.projects.iter().collect(),
    };
    for p in projects {
        let mut notes = read_project_notes(&cfg.settings.storage_dir, &p.folder_name, &p.id)?;
        all.append(&mut notes);
    }
    // Newest first for UI convenience
    all.reverse();
    Ok(all)
}

#[tauri::command]
pub fn save_note(
    app: AppHandle,
    state: State<'_, AppState>,
    input: AppendNoteInput,
) -> Result<Note, String> {
    let text = input.text.trim();
    if text.is_empty() {
        return Err("Note text is empty".into());
    }
    let mut cfg = state.config.lock();
    let project = find_project(&cfg, &input.project_id)?.clone();
    let mode = cfg.settings.reminders_mode.clone();
    let reminder = if let Some(snap) = input.reminder_snapshot {
        if !reminder_kind_allowed(&mode, &snap.kind) {
            return Err("Этот тип напоминаний отключён в настройках".into());
        }
        Some(snap)
    } else if let Some(ri) = &input.reminder {
        if !reminder_kind_allowed(&mode, &ri.kind) {
            return Err("Этот тип напоминаний отключён в настройках".into());
        }
        Some(build_reminder_from_input(ri)?)
    } else {
        None
    };
    let note = append_note(
        &cfg.settings.storage_dir,
        &project.folder_name,
        &project.id,
        text,
        reminder,
        input.closed,
        input.closed_at,
    )?;
    cfg.settings.last_project_id = Some(project.id.clone());
    cfg.draft = None;
    save_config(&cfg)?;
    let _ = app.emit("notes-changed", project.id.clone());
    let _ = app.emit("config-updated", cfg.clone());
    Ok(note)
}

#[tauri::command]
pub fn update_note(
    app: AppHandle,
    state: State<'_, AppState>,
    input: UpdateNoteInput,
) -> Result<Note, String> {
    let cfg = state.config.lock();
    let project = find_project(&cfg, &input.project_id)?;
    let mode = cfg.settings.reminders_mode.clone();
    let mut notes =
        read_project_notes(&cfg.settings.storage_dir, &project.folder_name, &project.id)?;
    let idx = notes
        .iter()
        .position(|n| n.id == input.note_id)
        .ok_or_else(|| "Note not found".to_string())?;
    notes[idx].body = input.text.trim_end().to_string();
    if input.set_reminder {
        notes[idx].reminder = match &input.reminder {
            Some(ri) => {
                if !reminder_kind_allowed(&mode, &ri.kind) {
                    return Err("Этот тип напоминаний отключён в настройках".into());
                }
                // Preserve status/notified when editing due of an existing reminder of same kind.
                let mut built = build_reminder_from_input(ri)?;
                if let Some(prev) = &notes[idx].reminder {
                    if prev.kind == built.kind
                        && prev.due_at == built.due_at
                        && prev.relative_days == built.relative_days
                    {
                        built.status = prev.status.clone();
                        built.snooze_until = prev.snooze_until.clone();
                        built.notified_at = prev.notified_at.clone();
                    }
                }
                Some(built)
            }
            None => None,
        };
    }
    let updated = notes[idx].clone();
    write_project_notes(&cfg.settings.storage_dir, &project.folder_name, &notes)?;
    let _ = app.emit("notes-changed", project.id.clone());
    Ok(updated)
}

#[tauri::command]
pub fn complete_reminder(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ReminderActionInput,
) -> Result<Note, String> {
    let cfg = state.config.lock();
    let project = find_project(&cfg, &input.project_id)?;
    let mut notes =
        read_project_notes(&cfg.settings.storage_dir, &project.folder_name, &project.id)?;
    {
        let note = find_note_mut(&mut notes, &input.note_id)?;
        mark_reminder_done(note)?;
    }
    let updated = notes
        .iter()
        .find(|n| n.id == input.note_id)
        .cloned()
        .ok_or_else(|| "Note not found".to_string())?;
    write_project_notes(&cfg.settings.storage_dir, &project.folder_name, &notes)?;
    let _ = app.emit("notes-changed", project.id.clone());
    let _ = app.emit("reminder-resolved", &input);
    Ok(updated)
}

#[tauri::command]
pub fn snooze_reminder(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SnoozeReminderInput,
) -> Result<Note, String> {
    let until = if let Some(u) = &input.until {
        crate::notes::normalize_local_datetime(u)?
    } else if let Some(mins) = input.minutes {
        if mins == 0 {
            return Err("Укажите длительность отложения".into());
        }
        snooze_until_from_minutes(mins)
    } else {
        snooze_until_tomorrow_morning()
    };

    let cfg = state.config.lock();
    let project = find_project(&cfg, &input.project_id)?;
    let mut notes =
        read_project_notes(&cfg.settings.storage_dir, &project.folder_name, &project.id)?;
    {
        let note = find_note_mut(&mut notes, &input.note_id)?;
        apply_snooze(note, until)?;
    }
    let updated = notes
        .iter()
        .find(|n| n.id == input.note_id)
        .cloned()
        .ok_or_else(|| "Note not found".to_string())?;
    write_project_notes(&cfg.settings.storage_dir, &project.folder_name, &notes)?;
    let _ = app.emit("notes-changed", project.id.clone());
    let _ = app.emit(
        "reminder-resolved",
        &ReminderActionInput {
            project_id: input.project_id.clone(),
            note_id: input.note_id.clone(),
        },
    );
    Ok(updated)
}

#[tauri::command]
pub fn set_note_closed(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SetNoteClosedInput,
) -> Result<Note, String> {
    let cfg = state.config.lock();
    let project = find_project(&cfg, &input.project_id)?;
    let mut notes =
        read_project_notes(&cfg.settings.storage_dir, &project.folder_name, &project.id)?;
    {
        let note = find_note_mut(&mut notes, &input.note_id)?;
        apply_note_closed(note, input.closed);
    }
    let updated = notes
        .iter()
        .find(|n| n.id == input.note_id)
        .cloned()
        .ok_or_else(|| "Note not found".to_string())?;
    write_project_notes(&cfg.settings.storage_dir, &project.folder_name, &notes)?;
    let _ = app.emit("notes-changed", project.id.clone());
    Ok(updated)
}

#[tauri::command]
pub fn delete_note(
    app: AppHandle,
    state: State<'_, AppState>,
    input: DeleteNoteInput,
) -> Result<Note, String> {
    let cfg = state.config.lock();
    let project = find_project(&cfg, &input.project_id)?;
    let mut notes =
        read_project_notes(&cfg.settings.storage_dir, &project.folder_name, &project.id)?;
    let idx = notes
        .iter()
        .position(|n| n.id == input.note_id)
        .ok_or_else(|| "Note not found".to_string())?;
    let removed = notes.remove(idx);
    // Reassign ids by index after rewrite
    for (i, n) in notes.iter_mut().enumerate() {
        n.id = format!("{}:{i}", project.id);
    }
    write_project_notes(&cfg.settings.storage_dir, &project.folder_name, &notes)?;
    let _ = app.emit("notes-changed", project.id.clone());
    Ok(removed)
}

#[tauri::command]
pub fn save_draft(
    state: State<'_, AppState>,
    draft: Option<Draft>,
) -> Result<(), String> {
    let mut cfg = state.config.lock();
    cfg.draft = draft.filter(|d| !d.text.trim().is_empty());
    save_config(&cfg)
}

#[tauri::command]
pub fn set_last_project(state: State<'_, AppState>, project_id: Option<String>) -> Result<(), String> {
    let mut cfg = state.config.lock();
    cfg.settings.last_project_id = project_id;
    save_config(&cfg)
}

#[tauri::command]
pub fn get_project_folder_path(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<String, String> {
    let cfg = state.config.lock();
    let project = find_project(&cfg, &project_id)?;
    Ok(
        project_notes_path(&cfg.settings.storage_dir, &project.folder_name)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
    )
}

#[tauri::command]
pub fn get_storage_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.config.lock().settings.storage_dir.clone())
}

#[tauri::command]
pub fn get_runtime_paths() -> Result<RuntimePaths, String> {
    Ok(runtime_paths())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocateDataRootInput {
    /// Absolute parent directory chosen by the user. Data goes to `{parent}/QuickNotes/data`.
    pub parent_path: String,
    /// When true, copy existing config/notes into the new folder (old folder is kept).
    pub copy_existing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocateDataRootResult {
    pub config: AppConfig,
    pub paths: RuntimePaths,
    pub migrated: bool,
}

#[tauri::command]
pub fn relocate_data_root(
    app: AppHandle,
    state: State<'_, AppState>,
    input: RelocateDataRootInput,
) -> Result<RelocateDataRootResult, String> {
    let parent = std::path::PathBuf::from(input.parent_path.trim());
    if parent.as_os_str().is_empty() {
        return Err("Выберите папку для данных".into());
    }
    let (_new_dir, cfg) =
        crate::data_root::relocate_data_parent(&parent, input.copy_existing)?;
    {
        let mut guard = state.config.lock();
        *guard = cfg.clone();
    }
    let paths = runtime_paths();
    let _ = app.emit("config-updated", &cfg);
    Ok(RelocateDataRootResult {
        config: cfg,
        paths,
        migrated: input.copy_existing,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetDataRootInput {
    pub copy_back: bool,
}

#[tauri::command]
pub fn reset_data_root(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ResetDataRootInput,
) -> Result<RelocateDataRootResult, String> {
    let (_new_dir, cfg) = crate::data_root::reset_data_parent_to_default(input.copy_back)?;
    {
        let mut guard = state.config.lock();
        *guard = cfg.clone();
    }
    let paths = runtime_paths();
    let _ = app.emit("config-updated", &cfg);
    Ok(RelocateDataRootResult {
        config: cfg,
        paths,
        migrated: input.copy_back,
    })
}

#[tauri::command]
pub async fn show_quick_window(app: AppHandle) -> Result<(), String> {
    crate::windows::show_quick(&app)
}

#[tauri::command]
pub async fn hide_quick_window(app: AppHandle) -> Result<(), String> {
    crate::windows::hide_quick(&app)
}

#[tauri::command]
pub async fn show_main_window(app: AppHandle, section: Option<String>) -> Result<(), String> {
    crate::windows::show_main(&app, section.as_deref())
}

#[tauri::command]
pub fn apply_glass_effect(app: AppHandle, enabled: bool) -> Result<(), String> {
    crate::windows::apply_glass(&app, enabled)
}

#[tauri::command]
pub fn reregister_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    crate::hotkey::register_hotkey(&app, &hotkey)
}
