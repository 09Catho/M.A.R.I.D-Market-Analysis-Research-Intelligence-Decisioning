use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GoogleCseClient {
    api_key: String,
    cx: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub items: Option<Vec<SearchItem>>,
    pub searchInformation: Option<SearchInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchItem {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchInfo {
    pub totalResults: String,
}

impl GoogleCseClient {
    pub fn new(api_key: String, cx: String) -> Self {
        Self {
            api_key,
            cx,
            client: reqwest::Client::new(),
        }
    }

    pub async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<SearchItem>> {
        let url = "https://www.googleapis.com/customsearch/v1";
        // Google CSE "num" param default is 10, max is 10.
        let num = limit.min(10).max(1);

        let resp = self.client.get(url)
            .query(&[("key", &self.api_key), ("cx", &self.cx), ("q", &query.to_string()), ("num", &num.to_string())])
            .send()
            .await?;

        let result: SearchResult = resp.json().await?;
        Ok(result.items.unwrap_or_default())
    }
}
