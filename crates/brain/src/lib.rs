use tb_tools::{ToolRunner, ToolOutput};
use tb_provider_ai::{LlmProvider, Message};
use sqlx::SqlitePool;
use std::sync::Arc;
use serde_json::Value;

pub struct Brain {
    tool_runner: Arc<ToolRunner>,
    llm: Box<dyn LlmProvider>,
    pool: SqlitePool,
}

impl Brain {
    pub fn new(tool_runner: Arc<ToolRunner>, llm: Box<dyn LlmProvider>, pool: SqlitePool) -> Self {
        Self { tool_runner, llm, pool }
    }

    pub async fn process_turn(&self, user_input: &str, session_id: &str) -> anyhow::Result<String> {
        // 1. Build Context (History, Tools, etc.)
        let tools = self.tool_runner.list_tools().await;
        let tool_desc = serde_json::to_string_pretty(&tools)?;

        let system_prompt = format!(
            r#"You are TermBrain, an AI research assistant.
Protocol TBP/1:
1. You interact with the user and tools.
2. Output valid JSON only.
3. If you need to use a tool, output: {{ "tool_call": {{ "id": "tool_id", "args": {{ ... }} }} }}
4. If you have a final answer, output: {{ "final_response": "..." }}
5. Available tools:
{}
6. Do not invent tools.
7. Current date: {}
"#,
            tool_desc,
            chrono::Utc::now().to_rfc3339()
        );

        let mut messages = vec![
            Message { role: "user".to_string(), content: user_input.to_string() }
        ];

        // Simple loop max 5 turns
        for _ in 0..5 {
            let response_text = self.llm.chat(messages.clone(), Some(system_prompt.clone())).await?;

            // Parse JSON
            // Handle if LLM wraps in ```json ... ```
            let clean_json = if let Some(start) = response_text.find("```json") {
                if let Some(end) = response_text[start..].find("```") {
                     // Finds the closing backticks? Need to be careful with indexing
                     // simplified:
                     response_text.replace("```json", "").replace("```", "").trim().to_string()
                } else {
                    response_text.replace("```json", "").trim().to_string()
                }
            } else {
                response_text.trim().to_string()
            };

            let parsed: Value = match serde_json::from_str(&clean_json) {
                Ok(v) => v,
                Err(_) => {
                    // If not JSON, assume final response text? Or error?
                    // Strict TBP/1 says "Output valid JSON only".
                    // Let's assume it's final response if it fails parsing but looks like text.
                    return Ok(clean_json);
                }
            };

            if let Some(final_resp) = parsed.get("final_response").and_then(|v| v.as_str()) {
                return Ok(final_resp.to_string());
            }

            if let Some(call) = parsed.get("tool_call") {
                let tool_id = call.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let args = call.get("args").cloned().unwrap_or(serde_json::json!({}));

                messages.push(Message { role: "assistant".to_string(), content: clean_json.clone() });

                let output = match self.tool_runner.run(tool_id, args).await {
                    Ok(o) => o,
                    Err(e) => ToolOutput {
                        summary: format!("Error: {}", e),
                        artifacts: vec![],
                        receipts: vec![],
                    }
                };

                let tool_result = serde_json::json!({
                    "tool_result": {
                        "id": tool_id,
                        "output": output.summary
                    }
                });

                messages.push(Message { role: "user".to_string(), content: tool_result.to_string() });
                continue;
            }

            // If neither, return as is
            return Ok(clean_json);
        }

        Ok("Max turns reached.".to_string())
    }
}
