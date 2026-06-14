//! Locating the user's shell and history file.

use std::path::PathBuf;

use super::Shell;

/// Best guess at the user's shell from `$SHELL`, defaulting to zsh.
pub fn detect_shell() -> Shell {
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.contains("fish") {
        Shell::Fish
    } else if shell.contains("bash") {
        Shell::Bash
    } else {
        Shell::Zsh
    }
}

/// Conventional history-file path for a shell, honoring `$HISTFILE` if set.
pub fn history_path(shell: Shell) -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("HISTFILE") {
        if !custom.is_empty() {
            return Some(PathBuf::from(custom));
        }
    }

    let home = dirs::home_dir()?;
    let path = match shell {
        Shell::Zsh => home.join(".zsh_history"),
        Shell::Bash => home.join(".bash_history"),
        Shell::Fish => home.join(".local/share/fish/fish_history"),
    };
    Some(path)
}
