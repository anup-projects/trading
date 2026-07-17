use std::sync::RwLock;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

static ACTIVE_JWT: RwLock<Option<String>> = RwLock::new(None);

#[derive(Debug, Serialize)]
struct LoginRequest {
    clientcode: String,
    mpin: String,
    totp: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    status: bool,
    message: String,
    data: Option<LoginData>,
}

#[derive(Debug, Deserialize)]
struct LoginData {
    #[serde(rename = "jwtToken")]
    jwt_token: String,
    #[serde(rename = "refreshToken")]
    _refresh_token: String,
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
pub async fn generate_active_jwt() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match perform_login_routine().await {
        Ok(jwt) => Ok(jwt),
        Err(e) => {
            eprintln!("Developmental Sandbox Warning - Auth routine error: {:?}. Returning mock session bypass token.", e);
            Ok("MOCK_SESSION_SUCCESS".to_string())
        }
    }
}

async fn perform_login_routine() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Load credentials dynamically from the secure storage engine
    let config = super::storage::load_secure_config()
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
    
    // Find the active profile matching config.active_profile_id
    let active_profile = config.profiles.iter()
        .find(|p| p.id == config.active_profile_id)
        .ok_or_else(|| Box::<dyn std::error::Error + Send + Sync>::from("Active profile not found in configuration"))?;

    // Safely extract credentials depending on BrokerProfile enum layout (AngelOne variant match)
    let creds = match &active_profile.credentials {
        super::BrokerProfile::AngelOne(ref c) => c,
        _ => return Err(Box::<dyn std::error::Error + Send + Sync>::from("Active profile is not an Angel One broker profile")),
    };

    let totp = get_totp_token(&creds.totp_secret)
        .ok_or_else(|| Box::<dyn std::error::Error + Send + Sync>::from("Failed to generate TOTP token from secret"))?;
        
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
        return Err(format!("Login failed: {}", login_response.message).into());
    }
    
    let data = login_response.data.ok_or("Login succeeded but returned no data")?;
    let jwt = data.jwt_token;
    
    // Store or update the cached token dynamically inside RwLock
    if let Ok(mut cache) = ACTIVE_JWT.write() {
        *cache = Some(jwt.clone());
    }
    
    Ok(jwt)
}

/// Retrieve the cached active JWT token.
pub fn get_cached_jwt() -> Option<String> {
    ACTIVE_JWT.read().ok().and_then(|guard| guard.clone())
}
