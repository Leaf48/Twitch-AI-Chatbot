use async_trait::async_trait;

use crate::chat_model::core::{ChatModel, ChatModelError, MessageRequest, MessageResponse};

pub struct OpenAi {}

#[async_trait]
impl ChatModel for OpenAi {
    async fn generate(&self, req: &MessageRequest) -> Result<MessageResponse, ChatModelError> {
        todo!()
    }
}
