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

pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn name(&self) -> String {
        "gemini".to_string()
    }

    async fn chat(&self, messages: Vec<Message>, system_prompt: Option<String>) -> anyhow::Result<String> {
        // Gemini API structure: https://ai.google.dev/api/rest/v1/models/generateContent
        // It uses "contents" array with "parts" and "role".
        // Roles: "user", "model". System instructions are passed differently in beta or just prepended.
        // For stable v1, system instructions can be tricky. v1beta supports systemInstruction.

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let mut contents = vec![];

        // Handle system prompt by prepending or using systemInstruction if supported.
        // Let's use system_instruction field for v1beta.
        let system_instruction = if let Some(sys) = system_prompt {
            Some(serde_json::json!({
                "parts": [{ "text": sys }]
            }))
        } else {
            None
        };

        for m in messages {
            let role = if m.role == "assistant" { "model" } else { "user" };
            contents.push(serde_json::json!({
                "role": role,
                "parts": [{ "text": m.content }]
            }));
        }

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": 0.0
            }
        });

        if let Some(sys) = system_instruction {
            body.as_object_mut().unwrap().insert("system_instruction".to_string(), sys);
        }

        let resp = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let txt = resp.text().await?;
            anyhow::bail!("Gemini Error: {}", txt);
        }

        let json: Value = resp.json().await?;
        // Path: candidates[0].content.parts[0].text
        let content = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

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
            "final_response": "This is a mock response because no AI provider is configured (OPENAI_API_KEY or GEMINI_API_KEY missing)."
        }).to_string())
    }
}
