use serde::{Deserialize, Serialize};

pub mod server;
pub mod client;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum Message {
    Request { id: u64, method: String, params: serde_json::Value },
    Response { id: u64, result: Option<serde_json::Value>, error: Option<String> },
    Event { topic: String, payload: serde_json::Value },
}

pub const DEFAULT_PORT: u16 = 9000;
