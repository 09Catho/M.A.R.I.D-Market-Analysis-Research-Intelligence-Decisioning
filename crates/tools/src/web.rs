use crate::{Tool, ToolSpec, ToolOutput, ToolContext, Receipt, Artifact};
use async_trait::async_trait;
use serde_json::{json, Value};
use tb_provider_search::GoogleCseClient;
use tb_provider_web::WebFetcher;
use sqlx::Row;
use chrono::Utc;

pub struct WebSearchTool {
    client: Option<GoogleCseClient>,
}

impl WebSearchTool {
    pub fn new(api_key: Option<String>, cx: Option<String>) -> Self {
        let client = if let (Some(k), Some(c)) = (api_key, cx) {
             Some(GoogleCseClient::new(k, c))
        } else {
             None
        };
        Self { client }
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            id: "web.search".to_string(),
            name: "Web Search".to_string(),
            description: "Search the internet using Google Custom Search".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 5 }
                },
                "required": ["query"]
            }),
            output_schema: json!({ "results": "array" }),
            tags: vec!["web".to_string(), "search".to_string()],
        }
    }

    async fn run(&self, args: Value, _ctx: &ToolContext) -> anyhow::Result<ToolOutput> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as u32;

        if self.client.is_none() {
            // Disabled behavior
             return Ok(ToolOutput {
                summary: "Web Search is disabled (missing GOOGLE_CSE_API_KEY or GOOGLE_CSE_CX)".to_string(),
                artifacts: vec![],
                receipts: vec![],
            });
        }

        let client = self.client.as_ref().unwrap();
        let items = client.search(query, limit).await?;

        let summary = format!("Found {} results for '{}'. Top result: {}", items.len(), query, items.first().map(|i| i.title.clone()).unwrap_or_default());

        let mut artifacts = vec![];
        let mut receipts = vec![];

        let json_content = serde_json::to_string_pretty(&items)?;

        artifacts.push(Artifact {
             id: format!("search-{}", Utc::now().timestamp()),
             kind: "search_results".to_string(),
             content: json_content.clone(),
        });

        receipts.push(Receipt {
            provider: "google_cse".to_string(),
            source: "api".to_string(),
            fetched_at: Utc::now().to_rfc3339(),
            details: json!({ "query": query, "count": items.len() }),
        });

        Ok(ToolOutput {
            summary,
            artifacts,
            receipts,
        })
    }
}

pub struct WebFetchTool {
    fetcher: WebFetcher,
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self { fetcher: WebFetcher::new() }
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            id: "web.fetch".to_string(),
            name: "Web Fetch".to_string(),
            description: "Fetch URL content, extract text, and cache it".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string" }
                },
                "required": ["url"]
            }),
            output_schema: json!({ "content": "string" }),
            tags: vec!["web".to_string()],
        }
    }

    async fn run(&self, args: Value, ctx: &ToolContext) -> anyhow::Result<ToolOutput> {
        let url = args.get("url").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("Missing url"))?;

        // Check cache first
        let cached = sqlx::query("SELECT content_text, title, fetched_at FROM web_pages WHERE url = ?")
            .bind(url)
            .fetch_optional(&ctx.pool)
            .await?;

        if let Some(row) = cached {
             let title: String = row.get("title"); // This might be empty if insertion failed to set it?
             let content: String = row.get("content_text");
             let fetched_at: String = row.get("fetched_at");

             return Ok(ToolOutput {
                 summary: format!("Fetched '{}' (cached)", title),
                 artifacts: vec![
                     Artifact { id: "content".to_string(), kind: "text".to_string(), content }
                 ],
                 receipts: vec![
                     Receipt {
                         provider: "web.cache".to_string(),
                         source: url.to_string(),
                         fetched_at: fetched_at,
                         details: json!({ "cached": true }),
                     }
                 ]
             });
        }

        // Fetch
        let page = self.fetcher.fetch(url).await?;

        // Cache
        sqlx::query(
            "INSERT OR REPLACE INTO web_pages (url, fetched_at, status_code, content_text, content_hash, canonical_url, title) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&page.url)
        .bind(&page.fetched_at)
        .bind(page.status as i32)
        .bind(&page.content_text)
        .bind(&page.content_hash)
        .bind(&page.url)
        .bind(&page.title)
        .execute(&ctx.pool)
        .await?;

        let bytes = page.content_text.len();

        Ok(ToolOutput {
             summary: format!("Fetched '{}' ({} bytes)", page.title, bytes),
             artifacts: vec![
                 Artifact { id: "content".to_string(), kind: "text".to_string(), content: page.content_text }
             ],
             receipts: vec![
                 Receipt {
                     provider: "web.fetch".to_string(),
                     source: page.url,
                     fetched_at: page.fetched_at,
                     details: json!({ "status": page.status, "bytes": bytes }),
                 }
             ]
        })
    }
}
