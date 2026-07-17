use std::time::Duration;
use tokio::time::interval;
use serde::Serialize;
use tauri::Emitter;

/// Structured pricing update frame for high-frequency stock metrics.
#[derive(Debug, Clone, Serialize)]
pub struct SimulatedTick {
    pub symbol: String,
    pub price: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub bid_price: f64,
    pub ask_price: f64,
    pub timestamp: u64,
}

/// Simulated market feed generator managing thread-isolated pricing simulation loops.
pub struct MockFeedEngine {
    app_handle: tauri::AppHandle,
}

impl MockFeedEngine {
    /// Creates a new instance of MockFeedEngine.
    ///
    /// # Parameters
    /// * `app_handle` - Tauri application handle used to dispatch payload frames.
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    /// Spawns a background thread task to run the asset simulation at 250ms intervals.
    pub fn start_simulation(self) {
        tokio::spawn(async move {
            let mut ticker_interval = interval(Duration::from_millis(250));
            let mut current_price = 3850.00;
            let o = current_price;
            let mut h = current_price;
            let mut l = current_price;
            let mut v_total = 100000;
            let mut seed = 123456789u64; // Low-latency pseudo-random seed to satisfy tree-shaking rule

            loop {
                ticker_interval.tick().await;

                // Linear congruential implementation for real-time asset paths
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let rand_val = (seed >> 32) as f64 / 4294967295.0;
                let price_change = (rand_val - 0.5) * 1.50;
                
                current_price = (current_price + price_change * 100.0).round() / 100.0;

                if current_price > h { h = current_price; }
                if current_price < l { l = current_price; }

                let spread = 0.20;
                let bid = current_price - (spread / 2.0);
                let ask = current_price + (spread / 2.0);
                let vol_tick = (rand_val * 200.0) as u64;
                v_total += vol_tick;

                let tick_payload = SimulatedTick {
                    symbol: "TCS".to_string(),
                    price: current_price,
                    open: o,
                    high: h,
                    low: l,
                    close: current_price,
                    volume: v_total,
                    bid_price: (bid * 100.0).round() / 100.0,
                    ask_price: (ask * 100.0).round() / 100.0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };

                // Stream tick to Presentation Layer via Tauri Inter-Process boundary
                let _ = self.app_handle.emit("mock-market-tick", &tick_payload);
            }
        });
    }
}

/// Helper function to start the simulation loop on startup.
pub async fn start_simulation_loop(app_handle: tauri::AppHandle) {
    let simulator = MockFeedEngine::new(app_handle);
    simulator.start_simulation();
}
