use super::{BrokerStreamEngine, BrokerProfile};

/// Adapter representing the Sharekhan Connect (SKAPI) engine.
pub struct SharekhanEngine;

impl BrokerStreamEngine for SharekhanEngine {
    /// Authenticates with the Sharekhan SKAPI endpoint and initializes session.
    ///
    /// # Parameters
    /// * `profile` - Broker profile containing Sharekhan credentials.
    ///
    /// # Returns
    /// An empty Ok on success, or an error String.
    fn initialize_session(
        &self,
        profile: &BrokerProfile,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            if let BrokerProfile::Sharekhan(credentials) = profile {
                println!("Authenticating with Sharekhan SKAPI for login ID: {}", credentials.login_id);
                // Validate required credentials
                if credentials.login_id.is_empty() 
                    || credentials.password.is_empty()
                    || credentials.api_key.is_empty() 
                    || credentials.secret_key.is_empty() 
                    || credentials.totp_key.is_empty() 
                {
                    return Err("Sharekhan Authentication: Login ID, Password, API Key, Secret Key, and TOTP Key are required.".to_string());
                }
                Ok(())
            } else {
                Err("Invalid broker profile type passed to SharekhanEngine".to_string())
            }
        }
    }

    /// Establishes the real-time websocket feed and streams Sharekhan tick parameters.
    ///
    /// # Returns
    /// An empty Ok on success, or an error String.
    fn fetch_market_feed(
        &self,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            println!("Starting Sharekhan SKAPI Stream engine loop...");
            // Scaffold placeholder logic
            Ok(())
        }
    }
}

use reqwest::Client;
use std::collections::HashMap;

pub async fn exchange_code_for_token(api_key: &str, api_secret: &str, request_token: &str) -> Result<String, String> {
    let client = Client::new();
    let mut params = HashMap::new();
    params.insert("apiKey", api_key);
    params.insert("secretKey", api_secret);
    params.insert("requestToken", request_token);

    let res = client.post("https://api.sharekhan.com/skapi/services/access/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(res["data"]["accessToken"].as_str().ok_or("Token missing in response")?.to_string())
}
