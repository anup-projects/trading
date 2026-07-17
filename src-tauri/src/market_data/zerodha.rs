use super::{BrokerStreamEngine, BrokerProfile};

/// Adapter representing the Zerodha Kite Connect engine.
pub struct ZerodhaEngine;

impl BrokerStreamEngine for ZerodhaEngine {
    /// Authenticates with the Zerodha Kite API endpoint and initializes session.
    ///
    /// # Parameters
    /// * `profile` - Broker profile containing Zerodha credentials.
    ///
    /// # Returns
    /// An empty Ok on success, or an error String.
    fn initialize_session(
        &self,
        profile: &BrokerProfile,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            if let BrokerProfile::Zerodha(credentials) = profile {
                println!("Authenticating with Zerodha Kite API for client ID: {}", credentials.client_id);
                // Validate required credentials
                if credentials.client_id.is_empty() || credentials.api_key.is_empty() || credentials.api_secret.is_empty() {
                    return Err("Zerodha Authentication: Client ID, API Key, and API Secret are required.".to_string());
                }
                Ok(())
            } else {
                Err("Invalid broker profile type passed to ZerodhaEngine".to_string())
            }
        }
    }

    /// Establishes the real-time websocket feed and streams Kite tick parameters.
    ///
    /// # Returns
    /// An empty Ok on success, or an error String.
    fn fetch_market_feed(
        &self,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            println!("Starting Zerodha Kite Stream engine loop...");
            // Scaffold placeholder logic
            Ok(())
        }
    }
}

use reqwest::Client;
use std::collections::HashMap;
use sha2::{Digest, Sha256};

fn calculate_checksum(api_key: &str, request_token: &str, api_secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}{}", api_key, request_token, api_secret).as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub async fn exchange_code_for_token(api_key: &str, api_secret: &str, request_token: &str) -> Result<String, String> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("api_key", api_key);
    params.insert("request_token", request_token);
    let checksum = calculate_checksum(api_key, request_token, api_secret);
    params.insert("checksum", &checksum);

    let res = client.post("https://api.kite.trade/session/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(res["data"]["access_token"].as_str().ok_or("Token missing in response")?.to_string())
}
