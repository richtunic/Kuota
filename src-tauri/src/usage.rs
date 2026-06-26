use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeeklyUsage {
    pub limit_requests: Option<i64>,
    pub remaining_requests: Option<i64>,
    pub resets_at: Option<String>,
    pub messages_sent: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HourlyUsage {
    pub limit_tokens: Option<i64>,
    pub remaining_tokens: Option<i64>,
    pub resets_at: Option<String>,
    pub messages_sent_5h: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedAccount {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub first_seen: Option<String>,
    pub last_request: Option<String>,
    pub weekly: Option<WeeklyUsage>,
    pub hourly: Option<HourlyUsage>,
    pub models: Option<HashMap<String, i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageState {
    pub accounts: HashMap<String, CapturedAccount>,
    pub last_updated: Option<String>,
}

fn usage_file_path() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".gptrouter").join("usage.json")
}

#[tauri::command]
pub fn read_usage() -> Result<UsageState, String> {
    let path = usage_file_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|_| "No hay datos aun. Activa el proxy y usa Codex.".to_string())?;

    serde_json::from_str(&content).map_err(|error| format!("Error parseando usage.json: {}", error))
}

pub fn start_polling(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_content = String::new();

        loop {
            if let Ok(content) = std::fs::read_to_string(usage_file_path()) {
                if content != last_content {
                    if let Ok(state) = serde_json::from_str::<UsageState>(&content) {
                        let _ = app.emit("usage-updated", state);
                    }
                    last_content = content;
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    });
}
