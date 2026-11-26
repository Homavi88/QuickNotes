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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub hotkey: String,
    pub autostart: bool,
    pub theme: Theme,
    pub accent_color: String,
    pub storage_dir: String,
    pub hide_on_blur: bool,
    pub glass_effect: bool,
    pub last_project_id: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteInput {
    pub project_id: String,
    pub note_id: String,
    pub text: String,
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
            autostart: false,
            theme: Theme::Dark,
            accent_color: "#7C5CFF".into(),
            storage_dir: default_storage_dir(),
            hide_on_blur: true,
            glass_effect: true,
            last_project_id: None,
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

/// Portable layout: `<exe_dir>/data/`. Installed layout: `%APPDATA%/QuickNotes`.
pub fn config_dir() -> PathBuf {
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
    RuntimePaths {
        portable: is_portable_mode(),
        exe_dir: exe.to_string_lossy().into_owned(),
        config_dir: config_dir().to_string_lossy().into_owned(),
        config_path: config_path().to_string_lossy().into_owned(),
        default_storage_dir: default_storage_dir(),
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
}
