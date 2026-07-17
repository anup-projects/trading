use serde::{Deserialize, Serialize};
use super::MarketTick;

#[derive(Debug, Serialize, Deserialize)]
pub struct YahooResponse {
    pub chart: ChartData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartData {
    pub result: Option<Vec<ChartResult>>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartResult {
    pub timestamp: Option<Vec<u64>>,
    pub indicators: Indicators,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Indicators {
    pub quote: Vec<Quote>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quote {
    pub open: Vec<Option<f64>>,
    pub high: Vec<Option<f64>>,
    pub low: Vec<Option<f64>>,
    pub close: Vec<Option<f64>>,
    pub volume: Vec<Option<u64>>,
}

/// Fetch historical market data from Yahoo Finance API.
pub async fn fetch_historical_data(ticker: &str, period: &str) -> Result<Vec<MarketTick>, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval=1d",
        ticker, period
    );
    
    let res = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;
        
    let payload: YahooResponse = res.json()
        .await
        .map_err(|e| format!("JSON parsing failed: {}", e))?;
        
    let result = payload.chart.result
        .ok_or_else(|| format!("No chart data returned: {:?}", payload.chart.error))?
        .into_iter()
        .next()
        .ok_or("Empty chart results")?;
        
    let timestamps = result.timestamp.unwrap_or_default();
    let quote = result.indicators.quote.into_iter().next()
        .ok_or("No quoting metrics found")?;
        
    let mut ticks = Vec::new();
    
    for i in 0..timestamps.len() {
        let t = timestamps[i];
        let o = quote.open.get(i).and_then(|x| *x);
        let h = quote.high.get(i).and_then(|x| *x);
        let l = quote.low.get(i).and_then(|x| *x);
        let c = quote.close.get(i).and_then(|x| *x);
        let v = quote.volume.get(i).and_then(|x| *x).unwrap_or(0);
        
        if let (Some(open), Some(high), Some(low), Some(close)) = (o, h, l, c) {
            ticks.push(MarketTick {
                timestamp: t,
                open,
                high,
                low,
                close,
                volume: v,
            });
        }
    }
    
    Ok(ticks)
}
