use std::time::Duration;
use sha2::{Sha256, Digest};
use readability::extractor;
use std::io::Cursor;
use url::Url;

#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub url: String,
    pub status: u16,
    pub content_text: String,
    pub title: String,
    pub content_hash: String,
    pub fetched_at: String,
}

#[derive(Clone)]
pub struct WebFetcher {
    client: reqwest::Client,
}

impl WebFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("TermBrain/1.0 (ResearchBot)")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn fetch(&self, url_str: &str) -> anyhow::Result<FetchedPage> {
        let url = Url::parse(url_str)?;
        // Simple host check to prevent localhost scanning if needed?
        if url.scheme() != "http" && url.scheme() != "https" {
            anyhow::bail!("Only HTTP/HTTPS allowed");
        }

        let resp = self.client.get(url.clone()).send().await?;
        let status = resp.status().as_u16();
        let bytes = resp.bytes().await?;

        let fetched_at = chrono::Utc::now().to_rfc3339();

        // Extract text using readability
        // Readability expects a Read, we wrap bytes in Cursor
        let mut reader = Cursor::new(&bytes);

        // This might fail for non-HTML.
        // For MVP, if it fails, fallback to raw text if possible or empty.
        let (title, content) = match extractor::extract(&mut reader, &url) {
             Ok(prod) => (prod.title, prod.text),
             Err(_) => {
                 // Fallback: try to convert bytes to string directly?
                 // Or just return error?
                 // Let's return partial info
                 (String::from("Unknown"), String::from_utf8_lossy(&bytes).to_string())
             }
        };

        // Hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = hex::encode(hasher.finalize());

        Ok(FetchedPage {
            url: url_str.to_string(),
            status,
            content_text: content,
            title,
            content_hash,
            fetched_at,
        })
    }
}
