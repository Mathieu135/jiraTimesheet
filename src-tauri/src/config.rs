use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub jira_url: String,
    pub email: String,
    pub api_token: String,
}

pub struct ConfigState {
    pub config: Mutex<AppConfig>,
}

impl ConfigState {
    pub fn new() -> Self {
        let config = AppConfig {
            jira_url: env::var("JIRA_URL").unwrap_or_default(),
            email: env::var("JIRA_EMAIL").unwrap_or_default(),
            api_token: env::var("JIRA_TOKEN").unwrap_or_default(),
        };

        Self {
            config: Mutex::new(config),
        }
    }
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, ConfigState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(
    state: tauri::State<'_, ConfigState>,
    jira_url: String,
    email: String,
    api_token: String,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = AppConfig {
        jira_url,
        email,
        api_token,
    };
    Ok(())
}
