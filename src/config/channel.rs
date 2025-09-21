use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use log::trace;
use once_cell::sync::Lazy;

use crate::config::{Account, CONFIG};

// account_name:channel, last_message_created_at
pub static CHANNELS: Lazy<Mutex<HashMap<String, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn init_channels() {
    for acc in &CONFIG.accounts {
        let k = format!("{}:{}", acc.account_name, acc.channel);

        let mut ch = CHANNELS.lock().unwrap();
        ch.insert(
            k,
            Instant::now() - Duration::from_secs(acc.interval.try_into().unwrap()),
        );
    }
}

/// Update value with current time
pub fn update_last_message_created_at(account: &Account) {
    CHANNELS.lock().unwrap().insert(
        format!("{}:{}", account.account_name, account.channel),
        Instant::now(),
    );
}

/// Get last message created at
pub fn get_last_message_created_at(account: &Account) -> Option<Instant> {
    let k = format!("{}:{}", account.account_name, account.channel);
    let mut ch = CHANNELS.lock().unwrap();
    ch.get(&k).copied()
}

/// Return true if account's interval exceeds elapsed time
pub fn can_send_now(account: &Account) -> bool {
    let key = format!("{}:{}", account.account_name, account.channel);

    let mut m = CHANNELS.lock().unwrap();

    let e = m.entry(key).or_insert_with(Instant::now);
    if e.elapsed() >= Duration::from_secs(account.interval.try_into().unwrap()) {
        *e = Instant::now();
        true
    } else {
        false
    }
}
