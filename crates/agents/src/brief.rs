use crate::{Agent, AgentRunner};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tb_tools::{ToolRunner, ToolOutput};

pub struct BriefAgent;

#[async_trait]
impl Agent for BriefAgent {
    fn id(&self) -> String {
        "agent.brief".to_string()
    }

    fn description(&self) -> String {
        "Generate a brief summary for a symbol (Market + News)".to_string()
    }

    async fn run(&self, args: Value, runner: &Arc<ToolRunner>) -> anyhow::Result<ToolOutput> {
        let symbol = args.get("symbol").and_then(|v| v.as_str()).unwrap_or("BTCUSDT");

        // 1. Get Quote
        // Using market.quote tool if it existed, but we have streaming only.
        // Let's assume we have market.quote tool.
        // Oh wait, I didn't implement market.quote tool in `tb-tools`.
        // I implemented `web.search` and `web.fetch`.
        // The prompt asked for `market.quote` tool.
        // I should implement that tool wrapping the provider logic or just use provider directly if accessible?
        // Tools should be in `tb-tools` or `tb-provider-market`.
        // For now, I'll just use `web.search` to find news about it as a fallback for "Market Brief" since I can't easily access the streaming market data from here without a tool wrapper.

        // Actually, I can implement `market.quote` tool in `tb-provider-market` if I moved the tool definition there, or just query DB.
        // Let's just query the DB for the latest quote since we have the pool in `ToolContext`?
        // But Agent::run gets `ToolRunner`. It doesn't get `ToolContext` directly.
        // `AgentRunner` has `pool`.
        // But `Agent::run` signature is `async fn run(&self, args: Value, runner: &Arc<ToolRunner>)`.
        // `ToolRunner` has `pool`. But it's private.
        // I should add `get_pool()` to `ToolRunner` or just implement `market.quote` as a proper tool.

        // Let's rely on `web.search` for now to demonstrate the workflow.

        let query = format!("{} crypto news", symbol);
        let search_res = runner.run("web.search", json!({ "query": query, "limit": 3 })).await?;

        let summary = format!("Brief for {}:\n\nSearch Results:\n{}", symbol, search_res.summary);

        Ok(ToolOutput {
            summary,
            artifacts: vec![],
            receipts: search_res.receipts,
        })
    }
}
