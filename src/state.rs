use dioxus::prelude::*;
use crate::{components::login::script::UserInfo, logger};

#[derive(Debug, Clone)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<UserInfo>,
}

pub static AUTH: GlobalSignal<AuthState> = Signal::global(|| AuthState {
    token: None,
    user: None,
});

pub static CONSOLE_LOG: GlobalSignal<Vec<logger::LogEntry>> = Signal::global(|| Vec::new());