use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub settings: Settings,
    pub projects: Vec<Project>,
    #[serde(default)]
    pub draft: Option<Draft>,
}

/// Runtime path layout (installed vs portable folder).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePaths {
    pub portable: bool,
    pub exe_dir: String,
    pub config_dir: String,
    pub config_path: String,
    pub default_storage_dir: String,
    /// User-chosen parent when data was relocated (`{parent}/QuickNotes/data`).
    /// `null` when using the built-in default layout.
    pub data_parent: Option<String>,
    /// Default parent used when no relocate pointer exists (exe dir or AppData parent).
    pub default_data_parent: String,
    /// Effective data folder that will be used for a given pending parent selection preview.
    pub data_dir_preview_suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub hotkey: String,
    /// In-window shortcut that saves the quick note (e.g. "Ctrl+Enter", "Enter").
    #[serde(default = "default_save_hotkey")]
    pub save_hotkey: String,
    pub autostart: bool,
    pub theme: Theme,
    pub accent_color: String,
    pub storage_dir: String,
    pub hide_on_blur: bool,
    /// When true, Esc / blur / hide with non-empty text saves to the selected project
    /// instead of only keeping a draft.
    #[serde(default)]
    pub save_on_hide: bool,
    pub glass_effect: bool,
    pub last_project_id: Option<String>,
    /// Which reminder kinds are allowed when creating/editing notes.
    #[serde(default)]
    pub reminders_mode: RemindersMode,
    /// Show OS / tray toast when a reminder is due.
    #[serde(default = "default_true")]
    pub reminder_notifications: bool,
}

fn default_true() -> bool {
    true
}

/// Master switch for reminder kinds available in the UI and due-checker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum RemindersMode {
    Off,
    Relative,
    Absolute,
    #[default]
    Both,
}

impl RemindersMode {
    pub fn allows(&self, kind: &ReminderKind) -> bool {
        match self {
            RemindersMode::Off => false,
            RemindersMode::Relative => matches!(kind, ReminderKind::Relative),
            RemindersMode::Absolute => matches!(kind, ReminderKind::Absolute),
            RemindersMode::Both => true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, RemindersMode::Off)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReminderKind {
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ReminderStatus {
    #[default]
    Pending,
    Done,
    Snoozed,
}

/// Reminder attached to a note (persisted in notes.md as an HTML comment).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteReminder {
    pub kind: ReminderKind,
    /// Local due time `YYYY-MM-DD HH:mm` (always stored absolute for checking).
    pub due_at: String,
    /// Present when kind is Relative — original “in N days” value for UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_days: Option<u32>,
    #[serde(default)]
    pub status: ReminderStatus,
    /// When status is Snoozed — next fire time `YYYY-MM-DD HH:mm`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snooze_until: Option<String>,
    /// Set after an OS/in-app notification was shown (avoids spam).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notified_at: Option<String>,
}

/// Input when creating/updating a reminder from the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderInput {
    pub kind: ReminderKind,
    /// For Relative: number of days from now (≥ 1).
    #[serde(default)]
    pub relative_days: Option<u32>,
    /// For Absolute: local datetime `YYYY-MM-DD HH:mm` or `YYYY-MM-DDTHH:mm`.
    #[serde(default)]
    pub absolute_at: Option<String>,
}

fn default_save_hotkey() -> String {
    "Ctrl+Enter".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub folder_name: String,
    pub color: String,
    pub emoji: String,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub text: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub project_id: String,
    pub timestamp: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reminder: Option<NoteReminder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub name: String,
    pub color: String,
    pub emoji: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderProjectsInput {
    pub ordered_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendNoteInput {
    pub project_id: String,
    pub text: String,
    #[serde(default)]
    pub reminder: Option<ReminderInput>,
    /// Exact reminder snapshot (e.g. undo delete). Takes precedence over `reminder`.
    #[serde(default)]
    pub reminder_snapshot: Option<NoteReminder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteInput {
    pub project_id: String,
    pub note_id: String,
    pub text: String,
    /// When `Some`, replaces the reminder (`None` inside clears it).
    /// When field omitted (`None` outer via Option default), leave unchanged —
    /// use `update_reminder` flag for clarity from the frontend.
    #[serde(default)]
    pub reminder: Option<ReminderInput>,
    /// If true, apply `reminder` (including clearing when None).
    #[serde(default)]
    pub set_reminder: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderActionInput {
    pub project_id: String,
    pub note_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnoozeReminderInput {
    pub project_id: String,
    pub note_id: String,
    /// Minutes from now (e.g. 15, 60). Ignored if `until` is set.
    #[serde(default)]
    pub minutes: Option<u32>,
    /// Absolute local `YYYY-MM-DD HH:mm`.
    #[serde(default)]
    pub until: Option<String>,
}

/// Payload emitted to the frontend when a reminder fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderDueEvent {
    pub project_id: String,
    pub note_id: String,
    pub project_name: String,
    pub body_preview: String,
    pub due_at: String,
    pub kind: ReminderKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNoteInput {
    pub project_id: String,
    pub note_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteProjectInput {
    pub id: String,
    pub delete_files: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Alt+N".into(),
            save_hotkey: default_save_hotkey(),
            autostart: false,
            theme: Theme::Dark,
            accent_color: "#7C5CFF".into(),
            storage_dir: default_storage_dir(),
            hide_on_blur: true,
            save_on_hide: false,
            glass_effect: true,
            last_project_id: None,
            reminders_mode: RemindersMode::Both,
            reminder_notifications: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            settings: Settings::default(),
            projects: Vec::new(),
            draft: None,
        }
    }
}

/// True when a `PORTABLE` marker file sits next to the executable.
/// Used by the unzip-and-run package so config/notes stay writable without install.
pub fn is_portable_mode() -> bool {
    static PORTABLE: OnceLock<bool> = OnceLock::new();
    *PORTABLE.get_or_init(|| {
        exe_dir()
            .map(|dir| dir.join("PORTABLE").is_file() || dir.join(".portable").is_file())
            .unwrap_or(false)
    })
}

pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

/// Effective config/data directory.
///
/// - Relocated: `{data-root pointer}/QuickNotes/data`
/// - Portable default: `<exe_dir>/data/`
/// - Installed default: `%APPDATA%/QuickNotes`
pub fn config_dir() -> PathBuf {
    if let Some(parent) = crate::data_root::read_data_parent_override() {
        return crate::data_root::data_dir_for_parent(&parent);
    }
    if is_portable_mode() {
        return exe_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("data");
    }
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("QuickNotes")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Portable default notes: `<exe_dir>/data/notes`.
/// Installed default: `%USERPROFILE%/QuickNotes`.
pub fn default_storage_dir() -> String {
    if is_portable_mode() {
        return config_dir()
            .join("notes")
            .to_string_lossy()
            .into_owned();
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("QuickNotes")
        .to_string_lossy()
        .into_owned()
}

pub fn runtime_paths() -> RuntimePaths {
    let exe = exe_dir().unwrap_or_else(|| PathBuf::from("."));
    let default_parent = crate::data_root::default_data_parent();
    RuntimePaths {
        portable: is_portable_mode(),
        exe_dir: exe.to_string_lossy().into_owned(),
        config_dir: config_dir().to_string_lossy().into_owned(),
        config_path: config_path().to_string_lossy().into_owned(),
        default_storage_dir: default_storage_dir(),
        data_parent: crate::data_root::current_data_parent()
            .map(|p| p.to_string_lossy().into_owned()),
        default_data_parent: default_parent.to_string_lossy().into_owned(),
        data_dir_preview_suffix: "QuickNotes/data".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_ends_with_config_json() {
        let path = config_path();
        assert_eq!(path.file_name().and_then(|s| s.to_str()), Some("config.json"));
    }

    #[test]
    fn settings_defaults_new_fields_from_legacy_json() {
        // r## so "#7C5CFF" does not terminate the raw string early
        let raw = r##"{
            "hotkey": "Ctrl+Alt+N",
            "autostart": false,
            "theme": "dark",
            "accentColor": "#7C5CFF",
            "storageDir": "C:/notes",
            "hideOnBlur": true,
            "glassEffect": true,
            "lastProjectId": null
        }"##;
        let s: Settings = serde_json::from_str(raw).expect("legacy settings parse");
        assert_eq!(s.save_hotkey, "Ctrl+Enter");
        assert!(!s.save_on_hide);
        assert_eq!(s.reminders_mode, RemindersMode::Both);
        assert!(s.reminder_notifications);
    }
}
