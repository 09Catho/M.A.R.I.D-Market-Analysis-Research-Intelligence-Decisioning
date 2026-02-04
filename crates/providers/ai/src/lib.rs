use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> String;
    async fn chat(&self, messages: Vec<Message>, system_prompt: Option<String>) -> anyhow::Result<String>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> String {
        "openai".to_string()
    }

    async fn chat(&self, messages: Vec<Message>, system_prompt: Option<String>) -> anyhow::Result<String> {
        let mut msgs = vec![];
        if let Some(sys) = system_prompt {
            msgs.push(serde_json::json!({ "role": "system", "content": sys }));
        }
        for m in messages {
            msgs.push(serde_json::json!({ "role": m.role, "content": m.content }));
        }

        let body = serde_json::json!({
            "model": self.model,
            "messages": msgs,
            "temperature": 0.0
        });

        let resp = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let txt = resp.text().await?;
            anyhow::bail!("OpenAI Error: {}", txt);
        }

        let json: Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        Ok(content)
    }
}

// Mock provider for when keys are missing
pub struct MockLlmProvider;

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn name(&self) -> String {
        "mock".to_string()
    }

    async fn chat(&self, _messages: Vec<Message>, _system_prompt: Option<String>) -> anyhow::Result<String> {
        Ok(serde_json::json!({
            "final_response": "This is a mock response because no AI provider is configured."
        }).to_string())
    }
}
