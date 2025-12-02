use dioxus::prelude::*;
use crate::components::login::script::UserInfo;

#[derive(Debug, Clone)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<UserInfo>,
}

pub static AUTH: GlobalSignal<AuthState> = Signal::global(|| AuthState {
    token: None,
    user: None,
});
