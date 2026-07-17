use std::collections::VecDeque;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tauri::Emitter;

use super::MarketTick;
use super::auth::{get_cached_jwt, generate_active_jwt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VwapTick {
    #[serde(flatten)]
    pub tick: MarketTick,
    pub rolling_vwap: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionPayload {
    pub action: u8, // 1 = Subscribe, 2 = Unsubscribe
    pub params: SubscriptionParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionParams {
    pub mode: u8, // 1 = Ltp, 2 = Quote, 3 = Snapquote
    #[serde(rename = "tokenList")]
    pub token_list: Vec<TokenItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenItem {
    #[serde(rename = "exchangeType")]
    pub exchange_type: u8, // 1 = NSE, 2 = BSE, etc.
    pub tokens: Vec<String>,
}

struct VwapTracker {
    window: VecDeque<(Instant, f64, u64)>,
}

impl VwapTracker {
    fn new() -> Self {
        Self {
            window: VecDeque::new(),
        }
    }

    fn add_tick(&mut self, price: f64, volume: u64) -> f64 {
        let now = Instant::now();
        self.window.push_back((now, price, volume));

        while let Some(front) = self.window.front() {
            if now.duration_since(front.0).as_secs() > 60 {
                self.window.pop_front();
            } else {
                break;
            }
        }

        let mut total_value = 0.0;
        let mut total_volume = 0;
        for &(_, p, v) in &self.window {
            total_value += p * (v as f64);
            total_volume += v;
        }

        if total_volume > 0 {
            total_value / (total_volume as f64)
        } else {
            price
        }
    }
}

static SUBSCRIPTION_TX: tokio::sync::OnceCell<mpsc::UnboundedSender<Vec<String>>> = tokio::sync::OnceCell::const_new();

/// Exposes a function to subscribe to dynamic token ticks.
pub fn subscribe_to_ticks(tokens: Vec<&str>) {
    if let Some(tx) = SUBSCRIPTION_TX.get() {
        let string_tokens = tokens.into_iter().map(|s| s.to_string()).collect();
        let _ = tx.send(string_tokens);
    }
}

/// Spawns the SmartStream WebSocket ingestion background thread.
pub async fn start_ingestion_loop(app_handle: tauri::AppHandle) {
    tokio::spawn(async move {
        let mut vwap_tracker = VwapTracker::new();
        let (sub_tx, mut sub_rx) = mpsc::unbounded_channel::<Vec<String>>();
        let _ = SUBSCRIPTION_TX.set(sub_tx);

        // Ensure we are logged in and have credentials
        if get_cached_jwt().is_none() {
            let _ = generate_active_jwt().await;
        }

        let jwt = match get_cached_jwt() {
            Some(token) => token,
            None => {
                eprintln!("Failed to run ingestion: No active JWT token cached.");
                return;
            }
        };

        let client_id = std::env::var("ANGEL_ONE_CLIENT_ID").unwrap_or_default();
        let api_key = std::env::var("ANGEL_ONE_API_KEY").unwrap_or_default();

        let ws_url = "wss://smartstream.smartapi.in/smartstream";
        let mut request = match ws_url.into_client_request() {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to construct request: {}", e);
                return;
            }
        };

        let headers = request.headers_mut();
        headers.insert("Authorization", reqwest::header::HeaderValue::from_str(&format!("Bearer {}", jwt)).unwrap());
        headers.insert("x-api-key", reqwest::header::HeaderValue::from_str(&api_key).unwrap());
        headers.insert("x-client-code", reqwest::header::HeaderValue::from_str(&client_id).unwrap());

        let ws_stream = match connect_async(request).await {
            Ok((stream, _)) => stream,
            Err(e) => {
                eprintln!("WebSocket connection failed: {}", e);
                return;
            }
        };

        let (mut write_half, mut read_half) = ws_stream.split();

        // Spawn a sender task to handle outgoing subscription commands
        tokio::spawn(async move {
            while let Some(tokens) = sub_rx.recv().await {
                let payload = SubscriptionPayload {
                    action: 1,
                    params: SubscriptionParams {
                        mode: 1,
                        token_list: vec![TokenItem {
                            exchange_type: 1,
                            tokens,
                        }],
                    },
                };
                if let Ok(json_str) = serde_json::to_string(&payload) {
                    let _ = write_half.send(Message::Text(json_str)).await;
                }
            }
        });

        // Read incoming binary packets
        while let Some(Ok(message)) = read_half.next().await {
            if let Message::Binary(bin) = message {
                // In production, parse the binary frame based on Angel One specifications.
                // For demonstration, we simulate parsing a binary packet.
                if bin.len() >= 26 {
                    let price = f64::from_be_bytes([bin[2], bin[3], bin[4], bin[5], bin[6], bin[7], bin[8], bin[9]]);
                    let volume = u64::from_be_bytes([bin[10], bin[11], bin[12], bin[13], bin[14], bin[15], bin[16], bin[17]]);
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

                    let tick = MarketTick {
                        timestamp,
                        open: price,
                        high: price,
                        low: price,
                        close: price,
                        volume,
                    };

                    let rolling_vwap = vwap_tracker.add_tick(price, volume);
                    let vwap_tick = VwapTick { tick, rolling_vwap };

                    let _ = app_handle.emit("market-tick", vwap_tick);
                }
            }
        }
    });
}

