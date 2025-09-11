use crate::{
    twitch::{Twitch, TwitchError},
    workflows::payload::MessagePayload,
};

pub async fn send_msg(payload: MessagePayload) -> Result<(), TwitchError> {
    let twitch = Twitch::new(payload.account);
    let ws = twitch.connect_to_chat().await?;
    let _ = twitch.send_chat(ws, payload.text).await?;

    Ok(())
}
