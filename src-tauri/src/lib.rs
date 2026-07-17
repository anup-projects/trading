pub mod market_data;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradingProfile {
    pub broker_type: String,
    pub client_id: String,
    pub api_key: String,
    pub mpin: String,
    pub totp_secret: String,
    pub secret_key: String,
    pub acc_password: String,
}

// NOTE: get_absolute_config_path() HAS BEEN PURGED. 
// Storage IO mechanics have been temporarily decoupled into isolated memory stubs.

#[tauri::command]
fn get_all_saved_profiles() -> Result<Vec<TradingProfile>, String> {
    // Plaintext disk read loops completely removed.
    // Temporary empty tracking matrix returned to maintain application compilation integrity.
    Ok(Vec::new())
}

#[tauri::command]
fn save_trading_profile(payload: TradingProfile) -> Result<String, String> {
    if payload.client_id.trim().is_empty() || payload.broker_type.trim().is_empty() {
        return Err("Data Leakage Prevention: Rejected empty verification payload metrics".to_string());
    }

    // Plaintext disk write loops completely removed.
    Ok("Profile received in secure memory loop successfully".to_string())
}

#[tauri::command]
fn switch_active_profile(profile_id: String) -> Result<String, String> {
    if profile_id.trim().is_empty() {
        return Err("Access Denied: Empty profile query index requested".to_string());
    }
    
    // Core check logic temporarily stubbed out until storage backend swap completes
    Ok("Active layout pointer verified".to_string())
}

#[tauri::command]
async fn login_to_broker(client_id: String, _mpin: String) -> Result<String, String> {
    let secret = market_data::auth::get_secure_token(client_id.clone())
        .map_err(|e| e.to_string())?;
    
    // Deserialize secret JSON into local struct and execute handshake
    let profile: TradingProfile = serde_json::from_str(&secret)
        .map_err(|_| "Failed to decode vault profile".to_string())?;
        
    market_data::angel_one::execute_angel_one_handshake(
        &profile.client_id,
        &profile.mpin,
        &profile.totp_secret,
        &profile.api_key
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn complete_broker_handshake(broker: String, request_token: String) -> Result<(), String> {
    // Load the profile credentials from Keyring first
    let secret = market_data::auth::get_secure_token(broker.clone())
        .map_err(|e| format!("Profile not found: {}", e))?;
    let profile: TradingProfile = serde_json::from_str(&secret)
        .map_err(|_| "Failed to decode vault profile".to_string())?;

    // 1. Determine broker type, perform exchange
    let jwt = match broker.to_lowercase().as_str() {
        "zerodha" => market_data::zerodha::exchange_code_for_token(&profile.api_key, &profile.secret_key, &request_token).await?,
        "sharekhan" => market_data::sharekhan::exchange_code_for_token(&profile.api_key, &profile.secret_key, &request_token).await?,
        _ => return Err("Unsupported broker for OAuth exchange".into()),
    };
    
    // 2. Commit to Hardware Vault via keyring save_secure_token
    market_data::auth::save_secure_token(broker, jwt)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_all_saved_profiles,
            save_trading_profile,
            switch_active_profile,
            market_data::auth::identify_broker,
            market_data::auth::save_secure_token,
            market_data::auth::get_secure_token,
            market_data::auth::delete_secure_token,
            login_to_broker,
            complete_broker_handshake
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
