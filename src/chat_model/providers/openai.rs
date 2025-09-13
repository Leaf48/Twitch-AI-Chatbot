use async_trait::async_trait;
use std::borrow::Cow;

use crate::{
    chat_model::core::{ChatModel, ChatModelError, MessageRequest, MessageResponse},
    config::CONFIG,
};

pub struct OpenAi {
    model: Cow<'static, str>,
}

impl OpenAi {
    pub fn new<S: Into<Cow<'static, str>>>(model: S) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait]
impl ChatModel for OpenAi {
    async fn generate(&self, req: &MessageRequest) -> Result<MessageResponse, ChatModelError> {
        // Read model
        let model = self.model.as_ref();

        // Build request payload for Chat Completions
        // { model, messages: [{role, content}, ...] }
        let messages_json: Vec<serde_json::Value> = req
            .messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": model,
            "messages": messages_json,
        });

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&CONFIG.openai.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let v: serde_json::Value = resp.json().await?;

        // Extract assistant message content
        let text = v
            .pointer("/choices/0/message/content")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ChatModelError::Api("missing choices[0].message.content".into()))?;

        // Extract usage tokens if present
        let used_tokens = v
            .pointer("/usage/total_tokens")
            .and_then(|x| x.as_u64())
            .map(|n| n as usize);

        Ok(MessageResponse { text, used_tokens })
    }
}
