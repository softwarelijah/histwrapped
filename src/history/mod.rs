//! Turns raw shell-history files into a clean `Vec<HistoryEntry>`.
//! Each shell parser lives in a submodule but produces the same type, so
//! everything downstream stays shell-agnostic.

pub mod bash;
pub mod detect;
pub mod fish;
pub mod zsh;

use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

/// A single command pulled from a history file. `timestamp` is optional
/// because some shells record no time information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: Option<DateTime<Local>>,
    pub shell: Shell,
}

impl HistoryEntry {
    /// The first token of the command, e.g. `git` for `git status -s`.
    pub fn program(&self) -> &str {
        self.command.split_whitespace().next().unwrap_or("")
    }
}
