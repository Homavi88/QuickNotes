use crate::models::AppConfig;
use parking_lot::Mutex;

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Mutex::new(config),
        }
    }
}
