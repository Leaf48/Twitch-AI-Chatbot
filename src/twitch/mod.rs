use futures_util::SinkExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::config::{Account, CONFIG};

#[derive(Clone)]
pub struct UserMessagePayload {
    pub account: Account,
}

#[derive(Debug, Error)]
pub enum TwitchError {
    #[error(transparent)]
    WebSocketError(#[from] tungstenite::Error),
}

pub struct Twitch {
    account: Account,
}

impl Twitch {
    pub fn new(account: Account) -> Self {
        Self { account }
    }

    pub async fn connect_to_chat(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, TwitchError> {
        let req = format!("wss://{}:443", CONFIG.twitch.host);
        let (mut ws, _resp) = connect_async(req).await?;

        ws.send(Message::Text(
            format!("PASS {}\r\n", self.account.oauth).into(),
        ))
        .await?;
        //* Nickname should not be random. It needs to be equal to oauth token's username.
        ws.send(Message::Text(
            format!("NICK {}\r\n", self.account.account_name).into(),
        ))
        .await?;
        ws.send(Message::Text(
            format!(
                "USER {} 8 * :{}\r\n",
                self.account.account_name, self.account.account_name
            )
            .into(),
        ))
        .await?;
        ws.send(Message::Text(
            format!("JOIN {}\r\n", self.account.channel).into(),
        ))
        .await?;

        Ok(ws)
    }

    pub async fn send_chat(
        &self,
        mut ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        text: String,
    ) -> Result<(), TwitchError> {
        ws.send(Message::Text(
            format!("PRIVMSG #{} :{}", self.account.account_name, text).into(),
        ))
        .await?;

        Ok(())
    }
}
