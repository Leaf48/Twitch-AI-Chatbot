use log::error;

use crate::{
    chat_model::{providers::openai::OpenAI, service::completion},
    config::Account,
    twitch::Twitch,
};

pub async fn recv_and_send_msg(account: &Account) {
    let twitch = Twitch::new(account);

    let ws = match twitch.connect_to_chat().await {
        Ok(ws) => ws,
        Err(err) => {
            error!("{:?}", err);
            return;
        }
    };

    let chats = match twitch.receive_chat(ws, account.chat_history_size).await {
        Ok(chats) => chats,
        Err(err) => {
            error!("{:?}", err);
            return;
        }
    };

    let openai = OpenAI::new(account.gpt_model.clone());

    let generated_msg = match completion::generate_chat(&chats, account, &openai).await {
        Ok(message) => message,
        Err(err) => {
            error!("{:?}", err);
            return;
        }
    };

    let ws = match twitch.connect_to_chat().await {
        Ok(ws) => ws,
        Err(err) => {
            error!("{:?}", err);
            return;
        }
    };

    if let Err(err) = twitch.send_chat(ws, generated_msg).await {
        error!("{:?}", err);
        return;
    }
}
