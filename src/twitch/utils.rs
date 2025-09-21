use log::error;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::config::Account;

pub async fn is_online(account: &Account) -> bool {
    let endpoint = "https://gql.twitch.tv/gql";

    let payload = json!(
    [
        {
            "operationName": "UseLive",
            "variables": {
                "channelLogin": account.channel
            },
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": "639d5f11bfb8bf3053b424d9ef650d04c4ebb7d94711d644afb08fe9a0fad5d9"
                }
            }
        }
    ]
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Client-id",
        "kimne78kx3ncx6brgo4mv6wki5h1ko".parse().unwrap(),
    );

    let payload_string = serde_json::to_string(&payload).unwrap();

    let client = reqwest::Client::new();
    let resp = match client
        .post(endpoint)
        .body(payload_string)
        .headers(headers)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            error!("{}", err);
            return false;
        }
    };

    if let Ok(json) = resp.json::<Value>().await {
        let stream = json.pointer("/0/data/user/stream");

        if let Some(obj) = stream {
            return !obj.is_null();
        }
    }

    false
}
