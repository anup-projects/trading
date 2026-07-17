pub mod auth;
pub mod angel_one;
pub mod zerodha;
pub mod sharekhan;
pub mod yahoo_fin;
pub mod mock_feed;

use serde::{Deserialize, Serialize};

/// Credentials specific to Angel One broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngelOneCredentials {
    /// Unique client identifier provided by Angel One
    pub client_id: String,
    /// API Developer Key used for signing/authenticating requests
    pub api_key: String,
    /// Mobile PIN used for secondary authentication
    pub mpin: String,
    /// Time-based One-Time Password secret key used for dynamic MFA generation
    pub totp_secret: String,
}

/// Credentials specific to Zerodha (Kite) broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZerodhaCredentials {
    /// Unique client identifier provided by Zerodha
    pub client_id: String,
    /// API Developer Key used for signing/authenticating requests
    pub api_key: String,
    /// API Secret used for generating access tokens
    pub api_secret: String,
}

/// Credentials specific to Sharekhan broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharekhanCredentials {
    /// Unique login identifier provided by Sharekhan
    pub login_id: String,
    /// Password used for primary authentication
    pub password: String,
    /// API Developer Key used for signing/authenticating requests
    pub api_key: String,
    /// Secret Key used for generating digital signatures
    pub secret_key: String,
    /// Time-based One-Time Password key used for MFA validation
    pub totp_key: String,
}

/// Unified enum representing the broker profile with specific credential signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "broker_type", content = "credentials")]
pub enum BrokerProfile {
    /// Angel One profile containing its specific credential signature
    AngelOne(AngelOneCredentials),
    /// Zerodha profile containing its specific credential signature
    Zerodha(ZerodhaCredentials),
    /// Sharekhan profile containing its specific credential signature
    Sharekhan(SharekhanCredentials),
}

/// Represents the credentials required to authenticate with any supported broker (legacy representation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    /// The name of the selected broker (e.g., "Angel One", "Zerodha", "Sharekhan")
    pub selected_broker: String,
    /// Unique client identifier provided by the broker
    pub client_id: String,
    /// API Developer Key used for signing/authenticating requests
    pub api_key: String,
    /// Mobile PIN or password password used for secondary authentication
    pub mpin: String,
    /// Time-based One-Time Password secret key used for dynamic MFA generation
    pub totp_secret: String,
}

/// Centered unified schema representing a single market tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// Defines a standardized engine interface for broker connectivity and streaming.
pub trait BrokerStreamEngine {
    /// Authenticates and initializes the client session with the broker using the provided BrokerProfile.
    ///
    /// # Parameters
    /// * `profile` - Reference to the broker profile credentials required for login.
    ///
    /// # Returns
    /// A result indicating success (`Ok(())`) or an error message (`Err(String)`).
    fn initialize_session(
        &self,
        profile: &BrokerProfile,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Connects to the broker's real-time tick stream feeds and fetches the market feed.
    ///
    /// # Returns
    /// A result indicating completion/failure details (`Result<(), String>`).
    fn fetch_market_feed(
        &self,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ScripItem {
    pub token: String,
    pub symbol: String,
    pub name: String,
    pub exch_seg: String,
}

#[tauri::command]
pub async fn get_broker_scrip_master() -> Result<Vec<ScripItem>, String> {
    let client = reqwest::Client::new();
    let response = client.get("https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json")
        .send()
        .await
        .map_err(|e| format!("Failed to reach Scrip Master: {}", e))?;

    let all_scrips: Vec<serde_json::Value> = response.json()
        .await
        .map_err(|e| format!("Failed to parse Scrip JSON: {}", e))?;

    // Architectural Guardrail: Filter down to active NSE Equity symbols in-memory to prevent UI layout bloating
    let filtered_scrips: Vec<ScripItem> = all_scrips.into_iter()
        .filter(|s| s["exch_seg"].as_str() == Some("NSE") && s["instrumenttype"].as_str() == Some(""))
        .take(500) // Cap the active UI hydration set for optimal render layout performance
        .map(|s| ScripItem {
            token: s["token"].as_str().unwrap_or_default().to_string(),
            symbol: s["symbol"].as_str().unwrap_or_default().to_string(),
            name: s["name"].as_str().unwrap_or_default().to_string(),
            exch_seg: s["exch_seg"].as_str().unwrap_or_default().to_string(),
        })
        .collect();

    Ok(filtered_scrips)
}

