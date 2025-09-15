use crate::config::Account;

pub struct MessagePayload<'a> {
    pub account: &'a Account,
    pub text: String,
}
