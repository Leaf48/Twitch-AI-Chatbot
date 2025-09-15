use log::error;

use crate::{twitch::Twitch, workflows::types::MessagePayload};

pub async fn send_msg(payload: MessagePayload<'_>) {
    let twitch = Twitch::new(payload.account);

    match twitch.connect_to_chat().await {
        Ok(ws) => {
            if let Err(err) = twitch.send_chat(ws, payload.text).await {
                error!("{:?}", err)
            }
        }
        Err(err) => {
            error!("{:?}", err)
        }
    }
}
