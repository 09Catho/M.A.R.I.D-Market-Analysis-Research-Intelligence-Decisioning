use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tb_tools::{ToolRunner, ToolOutput, ToolContext, ToolSpec, Tool};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

pub mod brief;

#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> String;
    fn description(&self) -> String;
    async fn run(&self, args: Value, runner: &Arc<ToolRunner>) -> anyhow::Result<ToolOutput>;
}

pub struct AgentRunner {
    registry: RwLock<HashMap<String, Box<dyn Agent>>>,
    tool_runner: Arc<ToolRunner>,
    pool: SqlitePool,
}

impl AgentRunner {
    pub fn new(tool_runner: Arc<ToolRunner>, pool: SqlitePool) -> Self {
        Self {
            registry: RwLock::new(HashMap::new()),
            tool_runner,
            pool,
        }
    }

    pub async fn register<A: Agent + 'static>(&self, agent: A) {
        self.registry.write().await.insert(agent.id(), Box::new(agent));
    }

    pub async fn run(&self, agent_id: &str, args: Value) -> anyhow::Result<ToolOutput> {
        let registry = self.registry.read().await;
        let agent = registry.get(agent_id).ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let run_id = Uuid::new_v4().to_string();
        let start_time = Utc::now().to_rfc3339();

        sqlx::query(
             "INSERT INTO agent_runs (agent_run_id, agent_id, started_at, input_json, status) VALUES (?, ?, ?, ?, 'running')"
        )
        .bind(&run_id)
        .bind(agent_id)
        .bind(&start_time)
        .bind(serde_json::to_string(&args)?)
        .execute(&self.pool)
        .await?;

        // Run
        let result = agent.run(args, &self.tool_runner).await;
        let end_time = Utc::now().to_rfc3339();

        match result {
             Ok(output) => {
                 sqlx::query(
                     "UPDATE agent_runs SET ended_at = ?, status = 'ok', output_summary_json = ?, receipts_json = ? WHERE agent_run_id = ?"
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
                 sqlx::query(
                     "UPDATE agent_runs SET ended_at = ?, status = 'error' WHERE agent_run_id = ?"
                 )
                 .bind(&end_time)
                 .bind(&run_id)
                 .execute(&self.pool)
                 .await?;
                 Err(e)
             }
        }
    }

    pub async fn list_agents(&self) -> Vec<(String, String)> {
        let registry = self.registry.read().await;
        registry.values().map(|a| (a.id(), a.description())).collect()
    }
}

// Meta Tools for Agents

pub struct SysAgentsList {
    runner: Arc<AgentRunner>,
}

impl SysAgentsList {
    pub fn new(runner: Arc<AgentRunner>) -> Self {
        Self { runner }
    }
}

#[async_trait]
impl Tool for SysAgentsList {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            id: "sys.agents.list".to_string(),
            name: "List Agents".to_string(),
            description: "List all available agents".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({ "agents": "array" }),
            tags: vec!["system".to_string()],
        }
    }

    async fn run(&self, _args: Value, _ctx: &ToolContext) -> anyhow::Result<ToolOutput> {
        let agents = self.runner.list_agents().await;
        let summary = format!("Found {} agents: {}", agents.len(), agents.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>().join(", "));

        Ok(ToolOutput {
            summary,
            artifacts: vec![],
            receipts: vec![],
        })
    }
}
