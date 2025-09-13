use async_trait::async_trait;
use thiserror::Error;

/// Message history
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Request payload
#[derive(Debug, Clone)]
pub struct MessageRequest {
    pub messages: Vec<Message>,
}

/// Completion response
#[derive(Debug, Clone)]
pub struct MessageResponse {
    pub text: String,
    pub used_tokens: Option<usize>,
}

/// ChatModel errors
#[derive(Error, Debug)]
pub enum ChatModelError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
}

/// Completion API traits
#[async_trait]
pub trait ChatModel {
    async fn generate(&self, req: &MessageRequest) -> Result<MessageResponse, ChatModelError>;
}
