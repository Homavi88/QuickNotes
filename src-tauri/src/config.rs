use crate::models::{config_dir, config_path, AppConfig};
use std::fs;
use std::io::Write;

pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();
    if !path.exists() {
        let cfg = AppConfig::default();
        save_config(&cfg)?;
        return Ok(cfg);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {e}"))?;
    let cfg: AppConfig =
        serde_json::from_str(&raw).map_err(|e| format!("Failed to parse config: {e}"))?;
    Ok(cfg)
}

pub fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
    let path = config_path();
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    {
        let mut f =
            fs::File::create(&tmp).map_err(|e| format!("Failed to write config temp: {e}"))?;
        f.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write config: {e}"))?;
        f.write_all(b"\n")
            .map_err(|e| format!("Failed to write config newline: {e}"))?;
    }
    fs::rename(&tmp, &path).map_err(|e| format!("Failed to finalize config: {e}"))?;
    Ok(())
}

pub fn unique_folder_name(storage_dir: &str, base: &str, exclude_id: Option<&str>, projects: &[crate::models::Project]) -> String {
    let mut candidate = base.to_string();
    let mut n = 2;
    loop {
        let clash = projects.iter().any(|p| {
            Some(p.id.as_str()) != exclude_id && p.folder_name.eq_ignore_ascii_case(&candidate)
        }) || {
            let path = std::path::Path::new(storage_dir).join(&candidate);
            path.exists()
                && !projects.iter().any(|p| {
                    Some(p.id.as_str()) == exclude_id && p.folder_name == candidate
                })
        };
        // Allow existing folder if it belongs to the same project being renamed.
        let owned = exclude_id.is_some()
            && projects
                .iter()
                .any(|p| Some(p.id.as_str()) == exclude_id && p.folder_name == candidate);
        if !clash || owned {
            return candidate;
        }
        candidate = format!("{base} ({n})");
        n += 1;
    }
}
