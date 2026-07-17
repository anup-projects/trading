use std::sync::RwLock;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use keyring::Entry;
use regex::Regex;

static ACTIVE_JWT: RwLock<Option<String>> = RwLock::new(None);

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerType {
    Zerodha,
    AngelOne,
    Sharekhan,
    Unknown,
}

/// System-level hook to identify broker based on ID structural footprint.
#[tauri::command]
pub fn identify_broker(login_id: String) -> BrokerType {
    let id = login_id.trim().to_uppercase();

    // Zerodha: Strictly 2 letters followed by 4 digits
    let zerodha_regex = Regex::new(r"^[A-Z]{2}\d{4}$").unwrap();
    
    // Angel One: 6 to 8 alphanumeric characters
    let angel_one_regex = Regex::new(r"^[A-Z0-9]{6,8}$").unwrap();

    // Sharekhan: 8-digit numeric ID
    let sharekhan_regex = Regex::new(r"^\d{8}$").unwrap();

    if zerodha_regex.is_match(&id) {
        BrokerType::Zerodha
    } else if sharekhan_regex.is_match(&id) {
        BrokerType::Sharekhan
    } else if angel_one_regex.is_match(&id) {
        BrokerType::AngelOne
    } else {
        BrokerType::Unknown
    }
}

// ============================================================================
// 1. ZERODHA (Kite Connect V3 Specs)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZerodhaSessionResponse {
    pub status: String,
    pub data: Option<ZerodhaSessionData>,
    pub error_type: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZerodhaSessionData {
    pub user_id: String,
    pub user_name: String,
    pub user_type: String,
    pub email: String,
    pub broker: String,
    pub access_token: String,
    pub public_token: String,
    pub login_time: String,
}

// ============================================================================
// 2. ANGEL ONE (SmartAPI Core V1 Specs)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngelOnePasswordResponse {
    pub status: bool,
    pub message: String,
    pub errorcode: String,
    pub data: Option<AngelOnePasswordData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngelOnePasswordData {
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngelOneOtpResponse {
    pub status: bool,
    pub message: String,
    pub errorcode: String,
    pub data: Option<AngelOneOtpData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngelOneOtpData {
    #[serde(rename = "jwtToken")]
    pub jwt_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "feedToken")]
    pub feed_token: String,
}

// ============================================================================
// 3. SHAREKHAN (SK API Gateway Specs)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharekhanTokenResponse {
    pub status: String,
    pub message: Option<String>,
    pub errorcode: Option<String>,
    #[serde(rename = "loginId")]
    pub login_id: Option<String>,
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub clientcode: String,
    pub mpin: String,
    pub totp: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<LoginData>,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    #[serde(rename = "jwtToken")]
    pub jwt_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Retrieves the specified broker authorization packet straight from the OS credential manager.
fn load_secure_config(client_id: &str) -> Result<String, String> {
    let vault_service = "com.nexus.trading.core";
    let profile_key = format!("profile_{}", client_id);
    
    // Bind to the authenticated platform key coordinates
    let entry = Entry::new(vault_service, &profile_key)
        .map_err(|e| format!("Vault Lookup Failure during broker auth handshake: {:?}", e))?;
        
    // Extract the raw secret payload bytes from kernel memory space
    let secret_bytes = entry.get_secret()
        .map_err(|e| format!("Access Denied: Broker credentials missing in OS vault matrix. {:?}", e))?;
        
    String::from_utf8(secret_bytes)
        .map_err(|e| format!("Payload Integrity Corruption Detected: {:?}", e))
}

/// Compute the active 6-digit TOTP token using the secret.
pub fn get_totp_token(secret: &str) -> Option<String> {
    let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret)?;
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();
    
    let code = totp_lite::totp_custom::<sha1::Sha1>(30, 6, &decoded, current_time);
    Some(code)
}

/// Automated login & active JWT generator module.
/// REMOVED: Mock session bypass. This now returns a strictly validated Result.
pub async fn generate_active_jwt() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let jwt = perform_login_routine().await?;
    Ok(jwt)
}

async fn perform_login_routine() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Retrieve the active profile identifier from the vault
    let active_entry = Entry::new("com.nexus.trading.core", "active_client_id")
        .map_err(|e| format!("Failed to access active profile key in keyring: {:?}", e))?;
    let active_bytes = active_entry.get_secret()
        .map_err(|e| format!("No active profile selected in keyring: {:?}", e))?;
    let client_id = String::from_utf8(active_bytes)
        .map_err(|e| format!("Active client ID string corruption: {:?}", e))?;

    // 2. Load the corresponding credentials from the secure platform vault
    let secret_json = load_secure_config(&client_id)
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
    
    let creds: crate::TradingProfile = serde_json::from_str(&secret_json)
        .map_err(|e| format!("Failed to parse profile JSON: {:?}", e))?;

    let totp = get_totp_token(&creds.totp_secret)
        .ok_or_else(|| Box::<dyn std::error::Error + Send + Sync>::from("Failed to generate TOTP token"))?;
        
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
        
    let login_payload = LoginRequest {
        clientcode: creds.client_id.clone(),
        mpin: creds.mpin.clone(),
        totp,
    };
    
    let response = client
        .post("https://smartapi.in/publisher-api/auth/verifylogin")
        .header("Content-Type", "application/json")
        .header("X-API-KEY", creds.api_key.clone())
        .json(&login_payload)
        .send()
        .await?;
        
    let login_response: LoginResponse = response.json().await?;
    if !login_response.status {
        // Enforcing strict failure propagation
        return Err(format!("Login failed: {}", login_response.message).into());
    }
    
    let data = login_response.data.ok_or("Login succeeded but returned no data")?;
    
    // Store or update the cached token dynamically inside RwLock
    if let Ok(mut cache) = ACTIVE_JWT.write() {
        *cache = Some(data.jwt_token.clone());
    }
    
    Ok(data.jwt_token)
}

/// Retrieve the cached active JWT token.
pub fn get_cached_jwt() -> Option<String> {
    ACTIVE_JWT.read().ok().and_then(|guard| guard.clone())
}
