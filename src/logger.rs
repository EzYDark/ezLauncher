use dioxus::prelude::*;
use colored::*;
use dioxus_core::Runtime;
use log::{Level, Metadata, Record, SetLoggerError};

use crate::state;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{}", record.args());
            
            let level_color = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn => "WARN".yellow(),
                Level::Info => "INFO".green(),
                Level::Debug => "DEBUG".blue(),
                Level::Trace => "TRACE".magenta(),
            };

            println!("[{}] {}", level_color, msg);
            
            // Only write to signal if we are in a Dioxus runtime
            if Runtime::try_current().is_some() {
                state::CONSOLE_LOG.write().push(LogEntry {
                    level: record.level(),
                    message: msg,
                });
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    colored::control::set_override(true);
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Debug))
}
