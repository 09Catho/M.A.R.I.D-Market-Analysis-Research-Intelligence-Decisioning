use std::time::Duration;
use tb_config::MarketProvider as MarketConfig;
use tb_ipc::Message;
use sqlx::{SqlitePool, Row};
use tokio::sync::broadcast;
use chrono::Utc;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub async fn start_market_service(
    config: MarketConfig,
    pool: SqlitePool,
    ipc_tx: broadcast::Sender<Message>,
) {
    let pool_clone = pool.clone();
    let ipc_clone = ipc_tx.clone();
    let config_clone = config.clone();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let interval = Duration::from_millis(config_clone.polling_interval_ms);

        loop {
            for symbol in &config_clone.symbols {
                // Fetch Quote
                if let Err(e) = fetch_quote(&client, symbol, &pool_clone, &ipc_clone).await {
                    tracing::error!("Quote fetch error for {}: {}", symbol, e);
                }
                // Fetch Bars
                if let Err(e) = fetch_bars(&client, symbol, "1m", &pool_clone, &ipc_clone).await {
                     tracing::error!("Bars fetch error for {}: {}", symbol, e);
                }
            }
            tokio::time::sleep(interval).await;
        }
    });
}

#[derive(serde::Deserialize, Debug)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

async fn fetch_quote(
    client: &reqwest::Client,
    symbol: &str,
    pool: &SqlitePool,
    ipc: &broadcast::Sender<Message>,
) -> anyhow::Result<()> {
    // Try binance.com, fallback to binance.us
    let url = format!("https://api.binance.us/api/v3/ticker/price?symbol={}", symbol);
    // Use timeout to fail fast
    let resp_res = client.get(&url).timeout(Duration::from_secs(2)).send().await;

    let text = match resp_res {
        Ok(resp) => resp.text().await?,
        Err(_) => String::from("timeout"),
    };

    // Check if error or timeout
    if text == "timeout" || (text.contains("\"code\"") && text.contains("\"msg\"")) {
        tracing::warn!("Binance API failed: {}. Using random walker stub.", text);
        return generate_mock_quote(symbol, pool, ipc).await;
    }

    let ticker: BinanceTicker = serde_json::from_str(&text)?;

    let price: f64 = ticker.price.parse()?;
    let price_i = (price * 100_000_000.0) as i64;
    let ts = Utc::now().to_rfc3339();

    let symbol_id = ensure_symbol(pool, symbol).await?;

    sqlx::query(
        "INSERT OR REPLACE INTO market_quotes (symbol_id, ts, last_i, source) VALUES (?, ?, ?, ?)"
    )
    .bind(symbol_id)
    .bind(&ts)
    .bind(price_i)
    .bind("binance")
    .execute(pool)
    .await?;

    // Broadcast
    let update = serde_json::json!({
        "symbol": symbol,
        "price": price,
        "ts": ts
    });

    let msg = Message::Event {
        topic: "market.quote".to_string(),
        payload: update,
    };
    let _ = ipc.send(msg);

    Ok(())
}

async fn fetch_bars(
    client: &reqwest::Client,
    symbol: &str,
    interval: &str,
    pool: &SqlitePool,
    ipc: &broadcast::Sender<Message>,
) -> anyhow::Result<()> {
    let url = format!("https://api.binance.us/api/v3/klines?symbol={}&interval={}&limit=50", symbol, interval);
    let resp_res = client.get(&url).timeout(Duration::from_secs(2)).send().await;

    let text = match resp_res {
        Ok(resp) => resp.text().await?,
        Err(_) => String::from("timeout"),
    };

    if text == "timeout" || (text.contains("\"code\"") && text.contains("\"msg\"")) {
         tracing::warn!("Binance API failed: {}. Using random walker stub.", text);
         return generate_mock_bars(symbol, interval, pool, ipc).await;
    }

    let items: Vec<serde_json::Value> = serde_json::from_str(&text)?;

    let symbol_id = ensure_symbol(pool, symbol).await?;

    let mut bars_data = Vec::new();

    for item in items {
        if let Some(arr) = item.as_array() {
            let ts_ms = arr[0].as_u64().unwrap_or(0);
            let ts = chrono::DateTime::from_timestamp_millis(ts_ms as i64).unwrap_or_default().to_rfc3339();

            let open: f64 = arr[1].as_str().unwrap_or("0").parse()?;
            let high: f64 = arr[2].as_str().unwrap_or("0").parse()?;
            let low: f64 = arr[3].as_str().unwrap_or("0").parse()?;
            let close: f64 = arr[4].as_str().unwrap_or("0").parse()?;
            let vol: f64 = arr[5].as_str().unwrap_or("0").parse()?;

            let open_i = (open * 100_000_000.0) as i64;
            let high_i = (high * 100_000_000.0) as i64;
            let low_i = (low * 100_000_000.0) as i64;
            let close_i = (close * 100_000_000.0) as i64;
            let vol_i = (vol * 100_000_000.0) as i64;

            sqlx::query(
                "INSERT OR REPLACE INTO market_bars (symbol_id, tf, ts, open_i, high_i, low_i, close_i, volume_i, source) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(symbol_id)
            .bind(interval)
            .bind(&ts)
            .bind(open_i)
            .bind(high_i)
            .bind(low_i)
            .bind(close_i)
            .bind(vol_i)
            .bind("binance")
            .execute(pool)
            .await?;

            bars_data.push(serde_json::json!({
                "ts": ts,
                "o": open,
                "h": high,
                "l": low,
                "c": close,
                "v": vol
            }));
        }
    }

    let msg = Message::Event {
        topic: format!("market.bars.{}.{}", symbol, interval),
        payload: serde_json::json!(bars_data),
    };
    let _ = ipc.send(msg);

    Ok(())
}

async fn ensure_symbol(pool: &SqlitePool, symbol: &str) -> anyhow::Result<i64> {
    // Attempt insert first (happy path for new symbols) or ignore
    // But we need the ID.
    // Use INSERT OR IGNORE, then SELECT.
    sqlx::query(
        "INSERT OR IGNORE INTO symbols (provider, symbol, display_name) VALUES ('binance', ?, ?)"
    )
    .bind(symbol)
    .bind(symbol)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT symbol_id FROM symbols WHERE symbol = ?")
        .bind(symbol)
        .fetch_one(pool)
        .await?;

    Ok(row.get("symbol_id"))
}

async fn generate_mock_quote(
    symbol: &str,
    pool: &SqlitePool,
    ipc: &broadcast::Sender<Message>,
) -> anyhow::Result<()> {
    // Use StdRng which is Send
    let mut rng = StdRng::from_entropy();
    let base = match symbol {
        "BTCUSDT" => 60000.0,
        "ETHUSDT" => 3000.0,
        "SOLUSDT" => 100.0,
        _ => 1000.0,
    };
    let variation = rng.gen_range(-0.01..0.01);
    let price = base * (1.0 + variation);

    let price_i = (price * 100_000_000.0) as i64;
    let ts = Utc::now().to_rfc3339();

    let symbol_id = ensure_symbol(pool, symbol).await?;

    sqlx::query(
        "INSERT OR REPLACE INTO market_quotes (symbol_id, ts, last_i, source) VALUES (?, ?, ?, ?)"
    )
    .bind(symbol_id)
    .bind(&ts)
    .bind(price_i)
    .bind("mock")
    .execute(pool)
    .await?;

    // Broadcast
    let update = serde_json::json!({
        "symbol": symbol,
        "price": price,
        "ts": ts
    });

    let msg = Message::Event {
        topic: "market.quote".to_string(),
        payload: update,
    };
    let _ = ipc.send(msg);

    Ok(())
}

async fn generate_mock_bars(
    symbol: &str,
    interval: &str,
    pool: &SqlitePool,
    ipc: &broadcast::Sender<Message>,
) -> anyhow::Result<()> {
    let mut rng = StdRng::from_entropy();
    let base = match symbol {
        "BTCUSDT" => 60000.0,
        "ETHUSDT" => 3000.0,
        "SOLUSDT" => 100.0,
        _ => 1000.0,
    };

    let now = Utc::now();
    let mut bars_data = Vec::new();
    let symbol_id = ensure_symbol(pool, symbol).await?;

    let mut current = base;

    for i in (0..50).rev() {
        let open: f64 = current;
        let change = rng.gen_range(-0.005..0.005);
        let close = open * (1.0 + change);
        let high = f64::max(open, close) * (1.0 + rng.gen_range(0.0..0.002));
        let low = f64::min(open, close) * (1.0 - rng.gen_range(0.0..0.002));
        let vol = rng.gen_range(100.0..1000.0);

        current = close;

        let ts = now - chrono::Duration::minutes(i);
        let ts_str = ts.to_rfc3339();

        let open_i = (open * 100_000_000.0) as i64;
        let high_i = (high * 100_000_000.0) as i64;
        let low_i = (low * 100_000_000.0) as i64;
        let close_i = (close * 100_000_000.0) as i64;
        let vol_i = (vol * 100_000_000.0) as i64;

         sqlx::query(
            "INSERT OR REPLACE INTO market_bars (symbol_id, tf, ts, open_i, high_i, low_i, close_i, volume_i, source) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(symbol_id)
        .bind(interval)
        .bind(&ts_str)
        .bind(open_i)
        .bind(high_i)
        .bind(low_i)
        .bind(close_i)
        .bind(vol_i)
        .bind("mock")
        .execute(pool)
        .await?;

        bars_data.push(serde_json::json!({
            "ts": ts_str,
            "o": open,
            "h": high,
            "l": low,
            "c": close,
            "v": vol
        }));
    }

    let msg = Message::Event {
        topic: format!("market.bars.{}.{}", symbol, interval),
        payload: serde_json::json!(bars_data),
    };
    let _ = ipc.send(msg);

    Ok(())
}
