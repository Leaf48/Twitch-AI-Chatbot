use log::info;
use Twitch_AI_Chatbot::{config::CONFIG, logger::LoggerSetup};

fn main() {
    LoggerSetup::new();
    info!("Available chatbots: {:?}", CONFIG.accounts.len());
}
