use crate::chat_model::core::ChatModelError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompletionError {
    #[error("Path resolve error: {0}")]
    PathResolve(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Chat model error: {0}")]
    ChatModel(#[from] ChatModelError),
}
