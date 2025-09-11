use crate::config::Account;

pub struct MessagePayload {
    pub account: Account,
    pub text: String,
}
