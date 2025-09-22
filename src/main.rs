use std::time::Duration;

use log::{debug, info, warn};

use tokio::time::{sleep, timeout};
use Twitch_AI_Chatbot::{
    config::{
        channel::{can_send_now, init_channels, update_last_message_created_at},
        OperatingMode, CONFIG,
    },
    logger::LoggerSetup,
    twitch::utils::is_online,
    workflows::recv_and_send_msg::recv_and_send_msg,
};

#[tokio::main]
async fn main() {
    LoggerSetup::new();
    info!("Available chatbots: {}", CONFIG.accounts.len());

    init_channels();

    loop {
        for account in &CONFIG.accounts {
            // check if interval time elapsed
            if !can_send_now(account) {
                continue;
            }

            let online_status = is_online(account).await;
            debug!(
                "Channel {} status: {} (operating mode: {:?})",
                account.channel,
                if online_status { "online" } else { "offline" },
                account.operating_mode
            );
            let should_process = match account.operating_mode {
                OperatingMode::ALWAYS => true,
                OperatingMode::OFFLINE => !online_status,
                OperatingMode::ONLINE => online_status,
            };
            if !should_process {
                update_last_message_created_at(account);
                continue;
            }

            // timeout if it exeeds set time.
            match timeout(
                Duration::from_secs(account.timeout.try_into().unwrap()),
                recv_and_send_msg(account),
            )
            .await
            {
                Ok(()) => info!("Completed message cycle for {}", account.channel),
                Err(_) => warn!(
                    "Channel {} timed out after {} seconds",
                    account.channel, account.timeout
                ),
            }

            // update hashmap
            update_last_message_created_at(account);
        }
        sleep(Duration::from_secs(1)).await;
    }
}
