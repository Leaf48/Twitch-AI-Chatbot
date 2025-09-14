use log::info;

use crate::{
    chat_model::{
        core::{ChatModel, Message, MessageRequest},
        service::types::CompletionError,
    },
    config::Account,
    twitch::UserMsg,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn generate_chat<T>(
    user_messages: &Vec<UserMsg>,
    account: &Account,
    completion_model: &T,
) -> Result<String, CompletionError>
where
    T: ChatModel,
{
    // Load template
    let instruction_path = resolve_instruction_path(&account.instruction).ok_or_else(|| {
        CompletionError::PathResolve(format!("unable to resolve path: {}", account.instruction))
    })?;

    // Read and parse template
    let json_string = fs::read_to_string(&instruction_path)?;
    let json: Vec<Message> = serde_json::from_str(&json_string)?;

    // Build placeholder map, see instructions/README.md
    let history = user_messages
        .iter()
        .filter(|m| m.sender != account.account_name)
        .map(|m| m.message.as_str())
        .collect::<Vec<_>>()
        .join(",");

    let mut placeholders: HashMap<&str, String> = HashMap::new();
    placeholders.insert("history", history);
    placeholders.insert("account_name", account.account_name.clone());
    placeholders.insert("channel", account.channel.clone());

    // Apply placeholders to instruction
    let messages: Vec<Message> = json
        .into_iter()
        .map(|mut m| {
            m.content = replace_placeholders(&m.content, &placeholders);
            m
        })
        .collect();

    // Build request and generate completion
    let req = MessageRequest { messages };
    let resp = completion_model.generate(&req).await?;

    info!(
        "answer: {} - used tokens: {}",
        resp.text,
        resp.used_tokens.unwrap_or_default()
    );

    Ok(resp.text)
}

fn resolve_instruction_path(instruction: &str) -> Option<PathBuf> {
    let p = Path::new(instruction);
    if p.is_absolute() || p.exists() {
        return Some(p.to_path_buf());
    }

    return None;
}

fn replace_placeholders(input: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars.iter() {
        let from = format!("{{{}}}", k);
        out = out.replace(&from, v);
    }
    out
}
