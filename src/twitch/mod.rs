use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use futures_util::{SinkExt, StreamExt};
use log::trace;
use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async_tls_with_config, connect_async,
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::config::{utils::get_account_names, Account, ProxyConfig, CONFIG};

pub mod utils;

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
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error("Proxy CONNECT failed: {0}")]
    ProxyConnectFailed(String),
    #[error("Proxy CONNECT response ended before headers completed")]
    ProxyResponseIncomplete,
    #[error("Invalid proxy configuration: {0}")]
    InvalidProxyConfig(&'static str),
}

pub struct Twitch<'a> {
    account: &'a Account,
}

impl<'a> Twitch<'a> {
    pub fn new(account: &'a Account) -> Self {
        Self { account }
    }

    pub async fn connect_to_chat(
        &self,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, TwitchError> {
        let req = format!("wss://{}:443", CONFIG.twitch.host);
        let (mut ws, _resp) = match &self.account.proxy {
            Some(proxy) => connect_via_proxy(&req, proxy).await?,
            None => connect_async(req.as_str()).await?,
        };

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
            format!("PRIVMSG #{} :{}", self.account.channel, text).into(),
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
        len: usize,
    ) -> Result<Vec<UserMsg>, TwitchError> {
        let mut msg_history: Vec<UserMsg> = Vec::new();

        while let Some(msg) = ws.next().await {
            let msg = msg?;
            if let Ok(text) = msg.to_text() {
                // send pong
                if text.starts_with("PING") {
                    ws.send(Message::Text("PONG\r\n".into())).await?;
                // PRIVMSG
                } else {
                    if let Some((sender, msg)) = parse_msg(text.trim()) {
                        trace!("{}: {}", sender, msg);

                        if get_account_names().contains(&sender.as_str()) {
                            continue;
                        }

                        msg_history.push(UserMsg {
                            sender: sender,
                            message: msg,
                        });
                    }
                }
            }

            if msg_history.len() >= len {
                break;
            }
        }

        Ok(msg_history)
    }
}

/// Make connection using proxy
async fn connect_via_proxy(
    request: &str,
    proxy: &ProxyConfig,
) -> Result<
    (
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tungstenite::handshake::client::Response,
    ),
    TwitchError,
> {
    // proxy url
    let proxy_url = Url::parse(&proxy.host)?;
    let host = proxy_url
        .host_str()
        .ok_or(TwitchError::InvalidProxyConfig("missing host"))?;
    let port = proxy_url
        .port_or_known_default()
        .ok_or(TwitchError::InvalidProxyConfig("missing port"))?;

    // connection to the proxy
    let mut stream = TcpStream::connect((host, port)).await?;

    // connection to Twitch IRC
    let twitch_irc = format!("{}:443", CONFIG.twitch.host);
    // craft CONNECT request
    let mut connect_request = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\n",
        twitch_irc, twitch_irc
    );

    // add credentials if username and password available
    if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
        let auth_value = build_proxy_authorization(&username, &password);
        connect_request.push_str(&format!("Proxy-Authorization: {}\r\n", auth_value));
    }

    connect_request.push_str("Proxy-Connection: keep-alive\r\n");
    connect_request.push_str("Connection: keep-alive\r\n\r\n");

    stream.write_all(connect_request.as_bytes()).await?;

    let response = read_proxy_connect_response(&mut stream).await?;
    let status_line = response.lines().next().unwrap_or("");
    let mut parts = status_line.split_whitespace();
    let _ = parts.next();
    let status_code = parts.next().unwrap_or("");
    if status_code != "200" {
        return Err(TwitchError::ProxyConnectFailed(status_line.to_string()));
    }

    client_async_tls_with_config(request, stream, None, None)
        .await
        .map_err(Into::into)
}

async fn read_proxy_connect_response(stream: &mut TcpStream) -> Result<String, TwitchError> {
    const MAX_RESPONSE_SIZE: usize = 8192;
    let mut response = Vec::with_capacity(256);
    let mut buf = [0u8; 1];

    while response.len() < MAX_RESPONSE_SIZE {
        let bytes_read = stream.read(&mut buf).await?;
        if bytes_read == 0 {
            break;
        }

        response.push(buf[0]);
        if response.len() >= 4 && &response[response.len() - 4..] == b"\r\n\r\n" {
            let text = String::from_utf8_lossy(&response).into_owned();
            return Ok(text);
        }
    }

    Err(TwitchError::ProxyResponseIncomplete)
}

fn build_proxy_authorization(username: &str, password: &str) -> String {
    let credentials = format!("{}:{}", username, password);

    format!("Basic {}", BASE64_STANDARD.encode(credentials))
}

/// Parse PRIVMSG and return (Sender, Message)
fn parse_msg(line: &str) -> Option<(String, String)> {
    let captures = MSG_RE.captures(line)?;

    let sender = captures.name("sender")?.as_str().to_string();
    let msg = captures.name("msg")?.as_str().to_string();

    Some((sender, msg))
}
