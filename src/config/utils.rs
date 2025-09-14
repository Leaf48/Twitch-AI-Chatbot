use crate::config::CONFIG;

/// Get list of chatbot name
pub fn get_account_names() -> Vec<&'static str> {
    CONFIG
        .accounts
        .iter()
        .map(|v| v.account_name.as_str())
        .collect::<Vec<_>>()
}
