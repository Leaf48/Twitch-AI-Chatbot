use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;

use crate::config::{Account, CONFIG};

// account_name:channel, next_executable_at
pub static CHANNELS: Lazy<Mutex<HashMap<String, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn init_channels() {
    let mut ch = CHANNELS.lock().unwrap();
    for acc in &CONFIG.accounts {
        ch.insert(channel_key(acc), Instant::now());
    }
}

fn channel_key(account: &Account) -> String {
    format!("{}:{}", account.account_name, account.channel)
}

/// Returns true if the current time is at or past the scheduled execution time.
pub fn can_execute(account: &Account) -> bool {
    let mut m = CHANNELS.lock().unwrap();
    let next_ready = m.entry(channel_key(account)).or_insert_with(Instant::now);
    Instant::now() >= *next_ready
}

/// Schedules the next execution by offsetting from now.
pub fn schedule_next_execution_in(account: &Account, offset_ms: Duration) {
    CHANNELS
        .lock()
        .unwrap()
        .insert(channel_key(account), Instant::now() + offset_ms);
}
