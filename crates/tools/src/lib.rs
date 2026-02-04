use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};

pub mod system;
pub mod web;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub summary: String,
    pub artifacts: Vec<Artifact>,
    pub receipts: Vec<Receipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub kind: String,
    pub content: String, // or path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub provider: String,
    pub source: String,
    pub fetched_at: String,
    pub details: Value,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn spec(&self) -> ToolSpec;
    async fn run(&self, args: Value, ctx: &ToolContext) -> anyhow::Result<ToolOutput>;
}

pub struct ToolContext {
    pub pool: SqlitePool,
}

pub struct ToolRunner {
    registry: RwLock<HashMap<String, Box<dyn Tool>>>,
    semaphores: HashMap<String, Arc<Semaphore>>,
    pool: SqlitePool,
}

impl ToolRunner {
    pub fn new(pool: SqlitePool) -> Self {
        let mut semaphores = HashMap::new();
        // Defaults: web=2, llm=2, market=5
        semaphores.insert("web".to_string(), Arc::new(Semaphore::new(2)));
        semaphores.insert("llm".to_string(), Arc::new(Semaphore::new(2)));
        semaphores.insert("market".to_string(), Arc::new(Semaphore::new(5)));
        semaphores.insert("default".to_string(), Arc::new(Semaphore::new(10)));

        Self {
            registry: RwLock::new(HashMap::new()),
            semaphores,
            pool,
        }
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        let spec = tool.spec();
        self.registry.write().await.insert(spec.id, Box::new(tool));
    }

    pub async fn run(&self, tool_id: &str, args: Value) -> anyhow::Result<ToolOutput> {
        let registry = self.registry.read().await;
        let tool = registry.get(tool_id).ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_id))?;
        let spec = tool.spec();

        // 1. Generic Caching Check
        // Hash(tool_id + args)
        let input_hash = {
            let mut hasher = Sha256::new();
            hasher.update(tool_id.as_bytes());
            hasher.update(serde_json::to_vec(&args)?);
            hex::encode(hasher.finalize())
        };

        // Check DB for recent valid run (generic cache)
        // We only cache if status='ok' and ended_at is recent (e.g. 1 hour).
        // Since sqlite dates are strings ISO8601, we can use datetime() comparisons.
        // For simplicity, let's just fetch the last run and check locally.
        let cached_run = sqlx::query(
            "SELECT output_summary_json, receipts_json, ended_at FROM tool_runs WHERE tool_id = ? AND input_json = ? AND status = 'ok' ORDER BY ended_at DESC LIMIT 1"
        )
        .bind(tool_id)
        .bind(serde_json::to_string(&args)?)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = cached_run {
            let ended_at: String = row.get("ended_at");
            // Parse ended_at
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ended_at) {
                let now = Utc::now();
                // 1 hour TTL default for generic tools.
                // Web Fetch has its own infinite cache logic inside the tool, but this layers on top.
                // Web Search specifically needs this.
                if now.signed_duration_since(dt).num_seconds() < 3600 {
                     let summary_json: String = row.get("output_summary_json");
                     let summary_obj: Value = serde_json::from_str(&summary_json)?;
                     let summary = summary_obj["summary"].as_str().unwrap_or("").to_string();

                     let receipts_json: String = row.get("receipts_json");
                     let mut receipts: Vec<Receipt> = serde_json::from_str(&receipts_json)?;

                     // Add generic cache receipt
                     receipts.push(Receipt {
                         provider: "toolrunner.cache".to_string(),
                         source: "db".to_string(),
                         fetched_at: Utc::now().to_rfc3339(),
                         details: serde_json::json!({ "original_run": ended_at, "ttl_remain": 3600 - now.signed_duration_since(dt).num_seconds() }),
                     });

                     return Ok(ToolOutput {
                         summary,
                         artifacts: vec![], // We don't hydrate artifacts from DB here yet, simplified.
                         receipts,
                     });
                }
            }
        }

        // 2. Concurrency Control (Semaphore)
        // Determine category from tags
        let category = if spec.tags.contains(&"web".to_string()) {
            "web"
        } else if spec.tags.contains(&"market".to_string()) {
            "market"
        } else {
            "default"
        };

        let sem = self.semaphores.get(category).unwrap_or_else(|| self.semaphores.get("default").unwrap());

        let wait_start = Utc::now();
        let _permit = sem.acquire().await?;
        let wait_duration_ms = (Utc::now() - wait_start).num_milliseconds();

        // 3. Execution
        let run_id = Uuid::new_v4().to_string();
        let start_time = Utc::now().to_rfc3339();

        // Log start
        sqlx::query(
            "INSERT INTO tool_runs (tool_run_id, tool_id, started_at, input_json, status) VALUES (?, ?, ?, ?, 'running')"
        )
        .bind(&run_id)
        .bind(tool_id)
        .bind(&start_time)
        .bind(serde_json::to_string(&args)?)
        .execute(&self.pool)
        .await?;

        let ctx = ToolContext {
            pool: self.pool.clone(),
        };

        // Run the tool
        let result = tool.run(args.clone(), &ctx).await;

        let end_time = Utc::now().to_rfc3339();

        match result {
            Ok(mut output) => {
                // Append metrics receipt
                output.receipts.push(Receipt {
                    provider: "toolrunner".to_string(),
                    source: "internal".to_string(),
                    fetched_at: end_time.clone(),
                    details: serde_json::json!({ "wait_ms": wait_duration_ms, "concurrency_category": category }),
                });

                // Log success
                sqlx::query(
                    "UPDATE tool_runs SET ended_at = ?, status = 'ok', output_summary_json = ?, receipts_json = ? WHERE tool_run_id = ?"
                )
                .bind(&end_time)
                .bind(serde_json::json!({ "summary": output.summary }).to_string())
                .bind(serde_json::to_string(&output.receipts)?)
                .bind(&run_id)
                .execute(&self.pool)
                .await?;

                Ok(output)
            }
            Err(e) => {
                // Log error
                sqlx::query(
                    "UPDATE tool_runs SET ended_at = ?, status = 'error', error_text = ? WHERE tool_run_id = ?"
                )
                .bind(&end_time)
                .bind(e.to_string())
                .bind(&run_id)
                .execute(&self.pool)
                .await?;

                Err(e)
            }
        }
    }

    pub async fn list_tools(&self) -> Vec<ToolSpec> {
        let registry = self.registry.read().await;
        registry.values().map(|t| t.spec()).collect()
    }
}
