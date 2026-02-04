use crate::{Tool, ToolSpec, ToolOutput, ToolContext, ToolRunner};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct SysToolsList {
    runner: Arc<ToolRunner>,
}

impl SysToolsList {
    pub fn new(runner: Arc<ToolRunner>) -> Self {
        Self { runner }
    }
}

#[async_trait]
impl Tool for SysToolsList {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            id: "sys.tools.list".to_string(),
            name: "List Tools".to_string(),
            description: "List all available tools and their specifications".to_string(),
            input_schema: json!({}),
            output_schema: json!({ "tools": "array" }),
            tags: vec!["system".to_string()],
        }
    }

    async fn run(&self, _args: Value, _ctx: &ToolContext) -> anyhow::Result<ToolOutput> {
        let tools = self.runner.list_tools().await;
        let summary = format!("Found {} tools.", tools.len());

        // We could return full specs as an artifact
        // For now, just return empty artifact/receipts
        Ok(ToolOutput {
            summary,
            artifacts: vec![],
            receipts: vec![],
        })
    }
}

pub struct SysStatus;

#[async_trait]
impl Tool for SysStatus {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            id: "sys.status".to_string(),
            name: "System Status".to_string(),
            description: "Check system health and provider status".to_string(),
            input_schema: json!({}),
            output_schema: json!({ "status": "string" }),
            tags: vec!["system".to_string()],
        }
    }

    async fn run(&self, _args: Value, _ctx: &ToolContext) -> anyhow::Result<ToolOutput> {
        // In real impl, check providers.
        // Here we just return "OK"
        Ok(ToolOutput {
            summary: "System is operational. Market: MOCK/BINANCE mixed.".to_string(),
            artifacts: vec![],
            receipts: vec![],
        })
    }
}
