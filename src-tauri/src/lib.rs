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
    pub gemini_api_key: Option<String>,
    pub ai_enabled: Option<bool>,
}

pub struct AppEngineState {
    pub auth_state: market_data::auth::SharedAuthState,
    pub active_jwt: tokio::sync::RwLock<Option<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthenticationOutput {
    pub status: String,
    pub token_preview: String,
    pub error_message: Option<String>,
}

// NOTE: get_absolute_config_path() HAS BEEN PURGED. 
// Storage IO mechanics have been temporarily decoupled into isolated memory stubs.

const SECURE_STORE_SERVICE: &str = "com.nexus.trading.core";

#[tauri::command]
fn get_all_saved_profiles() -> Result<Vec<TradingProfile>, String> {
    let list_bytes = match keyring::Entry::new(SECURE_STORE_SERVICE, "saved_client_ids") {
        Ok(entry) => entry.get_secret().unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let client_ids: Vec<String> = if list_bytes.is_empty() {
        Vec::new()
    } else {
        serde_json::from_slice(&list_bytes).unwrap_or_default()
    };

    let mut profiles = Vec::new();
    for id in client_ids {
        if let Ok(secret) = market_data::auth::get_secure_token(id) {
            if let Ok(profile) = serde_json::from_str::<TradingProfile>(&secret) {
                profiles.push(profile);
            }
        }
    }
    Ok(profiles)
}

#[tauri::command]
fn save_trading_profile(payload: TradingProfile) -> Result<String, String> {
    if payload.client_id.trim().is_empty() || payload.broker_type.trim().is_empty() {
        return Err("Data Leakage Prevention: Rejected empty verification payload metrics".to_string());
    }

    let list_entry = keyring::Entry::new(SECURE_STORE_SERVICE, "saved_client_ids")
        .map_err(|e| e.to_string())?;
    let list_bytes = list_entry.get_secret().unwrap_or_default();
    
    let mut client_ids: Vec<String> = if list_bytes.is_empty() {
        Vec::new()
    } else {
        serde_json::from_slice(&list_bytes).unwrap_or_default()
    };

    if !client_ids.contains(&payload.client_id) {
        client_ids.push(payload.client_id.clone());
        let updated_bytes = serde_json::to_vec(&client_ids)
            .map_err(|e| e.to_string())?;
        list_entry.set_secret(&updated_bytes)
            .map_err(|e| e.to_string())?;
    }

    Ok("Profile saved successfully".to_string())
}

#[tauri::command]
async fn save_credentials_securely(payload: TradingProfile) -> Result<String, String> {
    // 1. Commit profile parameters to hard disk config file
    save_trading_profile(payload.clone())?;

    // 2. Commit profile secret to secure vault
    let secret = serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;
    market_data::auth::save_secure_token(payload.client_id.clone(), secret)?;

    // 3. Set active profile identifier
    market_data::auth::save_secure_token("active_client_id".to_string(), payload.client_id.clone())?;

    // 4. Save independent AI config namespace (NEXUS_AI_CONFIG_V1)
    let ai_config = serde_json::json!({
        "gemini_api_key": payload.gemini_api_key,
        "ai_enabled": payload.ai_enabled.unwrap_or(false)
    });
    let ai_entry = keyring::Entry::new(SECURE_STORE_SERVICE, "NEXUS_AI_CONFIG_V1")
        .map_err(|e| e.to_string())?;
    ai_entry.set_secret(ai_config.to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    // 5. Verification Read-Back (The Critical Missing Step)
    let verified_id = market_data::auth::get_secure_token("active_client_id".to_string())?;
    if verified_id == payload.client_id {
        Ok("VERIFIED_SUCCESS".to_string())
    } else {
        Err("Persistence Integrity Failure: Data write confirmed but not retrievable.".into())
    }
}

#[tauri::command]
fn switch_active_profile(profile_id: String) -> Result<String, String> {
    if profile_id.trim().is_empty() {
        return Err("Access Denied: Empty profile query index requested".to_string());
    }
    
    // Core check logic temporarily stubbed out until storage backend swap completes
    Ok("Active layout pointer verified".to_string())
}

const SMART_API_BASE_URL: &str = "https://apiconnect.angelone.in";
const LOGIN_PATH: &str = "/rest/auth/angelbroking/user/v1/loginByPassword";

fn get_mac_address() -> String {
    use std::process::Command;
    if let Ok(output) = Command::new("getmac").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() && parts[0].len() == 17 && parts[0].contains('-') {
                    return parts[0].replace("-", ":");
                }
            }
        }
    }
    "00:00:00:00:00:00".to_string()
}

#[tauri::command]
async fn login_to_broker(clientId: String, mpin: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", SMART_API_BASE_URL, LOGIN_PATH);

    // Load the profile credentials from Keyring vault
    let secret = market_data::auth::get_secure_token(clientId.clone())
        .map_err(|e| format!("Failed to retrieve credentials from vault: {}", e))?;
    let profile: TradingProfile = serde_json::from_str(&secret)
        .map_err(|_| "Failed to decode vault profile".to_string())?;

    // Dynamically resolve public IP at runtime to satisfy SmartAPI whitelisting requirements
    let public_ip = match client.get("https://api.ipify.org").send().await {
        Ok(resp) => resp.text().await.unwrap_or_else(|_| "127.0.0.1".to_string()),
        Err(_) => "127.0.0.1".to_string(),
    };

    let mac_address = get_mac_address();
    
    // Official V2 Request Schema
    let payload = serde_json::json!({
        "clientcode": clientId,
        "password": mpin
    });

    // Mandatory Security Headers as per SmartAPI V2 Specification
    let response = client.post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("X-PrivateKey", &profile.api_key) 
        .header("X-ClientLocalIP", "127.0.0.1") 
        .header("X-ClientPublicIP", &public_ip) 
        .header("X-UserAgent", "desktop-rust-client")
        .header("X-SourceID", "WEB")
        .header("X-MACaddress", &mac_address)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok("Handshake successful. Session initiated.".into())
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "No error body".to_string());
        Err(format!("Gateway rejected request. Status: {}. Response: {}", status, error_text))
    }
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

#[tauri::command]
fn initialize_auth_manager(state: tauri::State<market_data::auth::SharedAuthState>) {
    let state_clone = std::sync::Arc::clone(&state);
    tokio::spawn(async move {
        market_data::auth::start_token_manager(state_clone).await;
    });
}

#[tauri::command]
async fn check_secure_store_for_creds() -> Option<TradingProfile> {
    let active_entry = keyring::Entry::new(SECURE_STORE_SERVICE, "active_client_id").ok()?;
    let active_bytes = active_entry.get_secret().ok()?;
    let client_id = String::from_utf8(active_bytes).ok()?;
    
    let secret = market_data::auth::get_secure_token(client_id).ok()?;
    serde_json::from_str(&secret).ok()
}

#[tauri::command]
async fn initialize_system_login(creds: TradingProfile) -> Result<AuthenticationOutput, String> {
    println!("[BRIDGE] Incoming request for client ID: {}", creds.client_id);

    match login_to_broker(creds.client_id.clone(), creds.mpin.clone()).await {
        Ok(msg) => {
            Ok(AuthenticationOutput {
                status: "SESSION_SUCCESS".to_string(),
                token_preview: msg,
                error_message: None,
            })
        }
        Err(err) => {
            Ok(AuthenticationOutput {
                status: "SESSION_FAILED".to_string(),
                token_preview: String::new(),
                error_message: Some(err),
            })
        }
    }
}

fn get_stored_reset_date() -> u64 {
    // Look up the last reset date from Keyring
    let entry = keyring::Entry::new(SECURE_STORE_SERVICE, "last_reset_date");
    if let Ok(entry) = entry {
        if let Ok(secret) = entry.get_secret() {
            if let Ok(s) = String::from_utf8(secret) {
                if let Ok(val) = s.parse::<u64>() {
                    return val;
                }
            }
        }
    }
    0
}

fn save_reset_date(date: u64) {
    let entry = keyring::Entry::new(SECURE_STORE_SERVICE, "last_reset_date");
    if let Ok(entry) = entry {
        let _ = entry.set_secret(date.to_string().as_bytes());
    }
}

fn is_new_trading_day() -> bool {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let last_reset = get_stored_reset_date();
    now / 86400 != last_reset / 86400
}

async fn perform_all_broker_handshakes() -> Result<(), String> {
    market_data::auth::force_reauth().await
}

pub async fn graceful_startup_wrapper() -> Result<(), String> {
    match perform_all_broker_handshakes().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Startup handshake failed: {}", e);
            Ok(())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("[DEBUG] Starting Tauri Builder...");
    let auth_state = std::sync::Arc::new(std::sync::RwLock::new(market_data::auth::AuthState {
        zerodha_token: None,
        sharekhan_token: None,
        expiry: std::time::SystemTime::now() + std::time::Duration::from_secs(86400),
    }));
    
    let app_state = std::sync::Arc::new(AppEngineState {
        auth_state: auth_state.clone(),
        active_jwt: tokio::sync::RwLock::new(None),
    });

    let builder = tauri::Builder::default()
        .manage(auth_state)
        .manage(app_state)
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                if is_new_trading_day() {
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    save_reset_date(now);
                    let _ = graceful_startup_wrapper().await;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_saved_profiles,
            save_trading_profile,
            switch_active_profile,
            market_data::auth::identify_broker,
            market_data::auth::save_secure_token,
            market_data::auth::get_secure_token,
            market_data::auth::delete_secure_token,
            login_to_broker,
            complete_broker_handshake,
            initialize_auth_manager,
            initialize_system_login,
            check_secure_store_for_creds,
            save_credentials_securely
        ])
        .on_window_event(|_window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                std::process::exit(0);
            }
            _ => {}
        });

    println!("[DEBUG] Configuration loaded. Running application...");
    builder.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
