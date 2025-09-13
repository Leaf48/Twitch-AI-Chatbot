use futures_util::{SinkExt, StreamExt};
use log::trace;
use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};

use crate::config::{Account, CONFIG};

// Regex of PRIVMSG
static MSG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^:(?P<sender>[^! ]+)![^ ]+@[^ ]+\.tmi\.twitch\.tv PRIVMSG #[^ ]+ :(?P<msg>.+)$")
        .unwrap()
});

#[derive(Debug, Clone)]
pub struct UserMsg {
    pub sender: String,
    pub message: String,
}

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
            format!("JOIN #{}\r\n", self.account.channel).into(),
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

    /// Receive chat
    ///
    /// retrieve: the number of messages sent by users to return
    pub async fn receive_chat(
        &self,
        mut ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        retrieve: usize,
    ) -> Result<Vec<UserMsg>, TwitchError> {
        let mut msg_history: Vec<UserMsg> = Vec::new();

        while let Some(msg) = ws.next().await {
            let msg = msg?;
            if let Ok(text) = msg.to_text() {
                // send pong
                if text.starts_with("PING") {
                    ws.send(Message::Text("PONG".into())).await?;
                // PRIVMSG
                } else {
                    if let Some((sender, msg)) = parse_msg(text.trim()) {
                        trace!("{}: {}", sender, msg);
                        msg_history.push(UserMsg {
                            sender: sender,
                            message: msg,
                        });
                    }
                }
            }

            if msg_history.len() >= retrieve {
                break;
            }
        }

        Ok(msg_history)
    }
}

/// Parse PRIVMSG and return (Sender, Message)
fn parse_msg(line: &str) -> Option<(String, String)> {
    let captures = MSG_RE.captures(line)?;

    let sender = captures.name("sender")?.as_str().to_string();
    let msg = captures.name("msg")?.as_str().to_string();

    Some((sender, msg))
}
