use log::info;
use Twitch_AI_Chatbot::logger::LoggerSetup;

fn main() {
    LoggerSetup::new();
    info!("Hello, World!");
}
