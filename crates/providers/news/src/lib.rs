use std::time::Duration;
use tb_config::NewsProvider as NewsConfig;
use tb_ipc::Message;
use sqlx::{SqlitePool, Row};
use tokio::sync::broadcast;
use chrono::Utc;
use rss::Channel;
use sha2::{Sha256, Digest};

pub async fn start_news_service(
    config: NewsConfig,
    pool: SqlitePool,
    ipc_tx: broadcast::Sender<Message>,
) {
    let pool_clone = pool.clone();
    let ipc_clone = ipc_tx.clone();
    let config_clone = config.clone();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let interval = Duration::from_secs(config_clone.fetch_interval_seconds);

        loop {
            for feed_url in &config_clone.feeds {
                if let Err(e) = fetch_feed(&client, feed_url, &pool_clone, &ipc_clone).await {
                    tracing::error!("Feed fetch error for {}: {}", feed_url, e);
                }
            }
            tokio::time::sleep(interval).await;
        }
    });
}

async fn fetch_feed(
    client: &reqwest::Client,
    url: &str,
    pool: &SqlitePool,
    ipc: &broadcast::Sender<Message>,
) -> anyhow::Result<()> {
    let content = client.get(url).timeout(Duration::from_secs(10)).send().await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    for item in channel.items {
        let title = item.title.unwrap_or_default();
        let link = item.link.unwrap_or_default();
        let pub_date = item.pub_date.unwrap_or_default();
        let description = item.description.unwrap_or_default();

        let exists = sqlx::query("SELECT 1 FROM news_items WHERE url = ?")
            .bind(&link)
            .fetch_optional(pool)
            .await?;

        if exists.is_some() {
            continue;
        }

        let published_at = match chrono::DateTime::parse_from_rfc2822(&pub_date) {
            Ok(dt) => dt.to_rfc3339(),
            Err(_) => Utc::now().to_rfc3339(),
        };
        let fetched_at = Utc::now().to_rfc3339();

        let mut hasher = Sha256::new();
        hasher.update(title.as_bytes());
        let content_hash = hex::encode(hasher.finalize());

        sqlx::query(
            "INSERT INTO news_items (provider, title, url, published_at, fetched_at, content_text, content_hash) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("rss")
        .bind(&title)
        .bind(&link)
        .bind(&published_at)
        .bind(&fetched_at)
        .bind(&description)
        .bind(&content_hash)
        .execute(pool)
        .await?;

        let msg = Message::Event {
            topic: "news.headline".to_string(),
            payload: serde_json::json!({
                "title": title,
                "url": link,
                "published_at": published_at,
                "source": channel.title
            }),
        };
        let _ = ipc.send(msg);
    }

    Ok(())
}
