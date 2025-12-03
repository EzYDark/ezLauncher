use anyhow::Result;
use dioxus::prelude::*;
use crate::{components::login::script::UserInfo, logger, scripts::app_dir::AppDir};

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

pub static APP_DIR: GlobalSignal<AppDir> = Signal::global(|| {
    let app = AppDir::new("ezLauncher");
    
    // Ensure App directory exists upon initialization
    if let Err(e) = app.ensure_exists() {
        log::error!("Failed to create App directory ({:?}):\n{}", app.path, e);
        std::process::exit(1);
    }
    
    app
});