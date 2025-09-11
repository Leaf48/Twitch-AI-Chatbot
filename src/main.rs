use log::info;
use Twitch_AI_Chatbot::{
    config::CONFIG,
    logger::LoggerSetup,
    twitch::{Twitch, UserMessagePayload},
    workflows::{model::MessagePayload, send_msg::send_msg},
};

#[tokio::main]
async fn main() {
    LoggerSetup::new();
    info!("Available chatbots: {:?}", CONFIG.accounts.len());

    let _ = send_msg(MessagePayload {
        account: CONFIG.accounts[0].clone(),
        text: "Hello World".to_string(),
    })
    .await;
}
