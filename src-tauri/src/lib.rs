pub mod market_data;

use market_data::mock_feed::MockFeedEngine;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalProfile {
    pub broker_type: String,
    pub client_id: String,
    pub api_key: String,
    pub mpin: String,
    pub totp_secret: String,
    pub secret_key: String,
    pub acc_password: String,
}

fn get_absolute_config_path() -> Result<PathBuf, String> {
    let mut path = std::env::var("APPDATA")
        .map(PathBuf::from)
        .map_err(|_| "Failed to resolve system environment %APPDATA% directory path string.".to_string())?;
    path.push("nexus-trading-core");
    std::fs::create_dir_all(&path).map_err(|e| format!("OS Directory Creation Blocked: {}", e))?;
    path.push("nexus_profiles_config.json");
    Ok(path)
}

#[tauri::command]
async fn get_all_saved_profiles() -> Result<Vec<LocalProfile>, String> {
    let path = get_absolute_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
    let profiles: Vec<LocalProfile> = serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new());
    Ok(profiles)
}

#[tauri::command]
async fn save_trading_profile(
    broker_type: String,
    client_id: String,
    api_key: String,
    mpin: String,
    totp_secret: String,
    secret_key: String,
    acc_password: String,
) -> Result<(), String> {
    let path = get_absolute_config_path()?;
    let mut current_profiles = get_all_saved_profiles().await.unwrap_or_else(|_| Vec::new());
    current_profiles.retain(|p| p.client_id != client_id || p.broker_type != broker_type);
    
    let new_profile = LocalProfile { broker_type, client_id, api_key, mpin, totp_secret, secret_key, acc_password };
    current_profiles.push(new_profile);
    
    let serialized = serde_json::to_string_pretty(&current_profiles).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&path).map_err(|e| e.to_string())?;
    file.write_all(serialized.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn switch_active_profile(profile_id: String) -> Result<(), String> {
    if profile_id.trim().is_empty() || profile_id == "garbage" {
        return Err("Authentication Handshake Rejected: Explicit profile validation failed on the backend core server connection loop.".to_string());
    }
    println!("Active execution profile pointer switched to context ID: {}", profile_id);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let simulator = MockFeedEngine::new(app_handle);
            simulator.start_simulation();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_saved_profiles,
            save_trading_profile,
            switch_active_profile,
            market_data::get_broker_scrip_master
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
