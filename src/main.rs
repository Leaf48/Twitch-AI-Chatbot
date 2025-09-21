use std::time::Duration;

use log::{debug, info, warn};

use tokio::time::{sleep, timeout};
use Twitch_AI_Chatbot::{
    config::{
        channel::{can_send_now, init_channels},
        CONFIG,
    },
    logger::LoggerSetup,
    workflows::recv_and_send_msg::recv_and_send_msg,
};

#[tokio::main]
async fn main() {
    LoggerSetup::new();
    info!("Available chatbots: {:?}", CONFIG.accounts.len());

    init_channels();

    loop {
        for account in &CONFIG.accounts {
            // check if interval time elapsed
            if !can_send_now(account) {
                continue;
            }

            // timeout if it exeeds set time.
            match timeout(
                Duration::from_secs(account.timeout.try_into().unwrap()),
                recv_and_send_msg(account),
            )
            .await
            {
                Ok(()) => debug!("completed: {}", account.account_name),
                Err(_) => warn!("timeout: {} secs", account.timeout),
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}
