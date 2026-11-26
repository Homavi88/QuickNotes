use crate::models::{
    Note, ReminderDueEvent, ReminderKind, ReminderStatus, RemindersMode,
};
use crate::notes::{
    effective_due, format_now_local, parse_local_datetime, read_project_notes, write_project_notes,
};
use crate::startup;
use crate::state::AppState;
use crate::windows::show_main;
use chrono::Local;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

const POLL_SECS: u64 = 30;

/// Background poller: finds due reminders, notifies once, emits `reminder-due`.
pub fn start_reminder_loop(app: AppHandle) {
    thread::spawn(move || {
        startup::log_line("reminders: poller started");
        loop {
            if let Err(e) = tick(&app) {
                startup::log_line(&format!("reminders: tick error: {e}"));
            }
            thread::sleep(Duration::from_secs(POLL_SECS));
        }
    });
}

fn tick(app: &AppHandle) -> Result<(), String> {
    let Some(state) = app.try_state::<AppState>() else {
        return Ok(());
    };
    let cfg = state.config.lock().clone();
    if !cfg.settings.reminders_mode.is_enabled() {
        return Ok(());
    }
    let mode = cfg.settings.reminders_mode.clone();
    let notifications = cfg.settings.reminder_notifications;
    let storage = cfg.settings.storage_dir.clone();
    let now = Local::now().naive_local();

    for project in &cfg.projects {
        let mut notes = read_project_notes(&storage, &project.folder_name, &project.id)?;
        let mut dirty = false;
        for note in &mut notes {
            let Some(reminder) = note.reminder.as_mut() else {
                continue;
            };
            if matches!(reminder.status, ReminderStatus::Done) {
                continue;
            }
            if !mode.allows(&reminder.kind) {
                continue;
            }
            // notified_at is cleared on snooze so the snooze time can fire once.
            if reminder.notified_at.is_some() {
                continue;
            }

            let due_str = effective_due(reminder);
            let Ok(due) = parse_local_datetime(due_str) else {
                continue;
            };
            if due > now {
                continue;
            }

            let preview: String = note
                .body
                .chars()
                .take(120)
                .collect::<String>()
                .replace('\n', " ");
            let event = ReminderDueEvent {
                project_id: project.id.clone(),
                note_id: note.id.clone(),
                project_name: format!("{} {}", project.emoji, project.name),
                body_preview: preview.clone(),
                due_at: due_str.to_string(),
                kind: reminder.kind.clone(),
            };

            if notifications {
                let title = format!("QuickNotes — {}", project.name);
                let body = if preview.is_empty() {
                    "Напоминание по заметке".into()
                } else {
                    preview.clone()
                };
                if let Err(e) = app
                    .notification()
                    .builder()
                    .title(&title)
                    .body(&body)
                    .show()
                {
                    startup::log_line(&format!("reminders: toast failed: {e}"));
                }
            }

            let _ = app.emit("reminder-due", &event);
            // Bring main window forward lightly so Done/Snooze UI is reachable.
            let _ = show_main(app, Some("notes"));

            reminder.notified_at = Some(format_now_local());
            if matches!(reminder.status, ReminderStatus::Snoozed) {
                reminder.status = ReminderStatus::Pending;
                reminder.snooze_until = None;
            }
            dirty = true;
        }
        if dirty {
            write_project_notes(&storage, &project.folder_name, &notes)?;
            let _ = app.emit("notes-changed", project.id.clone());
        }
    }
    Ok(())
}

pub fn find_note_mut<'a>(
    notes: &'a mut [Note],
    note_id: &str,
) -> Result<&'a mut Note, String> {
    notes
        .iter_mut()
        .find(|n| n.id == note_id)
        .ok_or_else(|| "Note not found".to_string())
}

pub fn mark_reminder_done(note: &mut Note) -> Result<(), String> {
    let reminder = note
        .reminder
        .as_mut()
        .ok_or_else(|| "У заметки нет напоминания".to_string())?;
    reminder.status = ReminderStatus::Done;
    reminder.snooze_until = None;
    reminder.notified_at = Some(format_now_local());
    Ok(())
}

pub fn apply_snooze(note: &mut Note, until: String) -> Result<(), String> {
    let reminder = note
        .reminder
        .as_mut()
        .ok_or_else(|| "У заметки нет напоминания".to_string())?;
    let _ = parse_local_datetime(&until)?;
    reminder.status = ReminderStatus::Snoozed;
    reminder.snooze_until = Some(until);
    reminder.notified_at = None; // allow fire again at snooze time
    Ok(())
}

pub fn reminder_kind_allowed(mode: &RemindersMode, kind: &ReminderKind) -> bool {
    mode.allows(kind)
}
