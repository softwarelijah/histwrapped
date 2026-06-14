//! Parser for bash history (`~/.bash_history`).
//!
//! By default bash stores one raw command per line with no timestamps. If the
//! user set `HISTTIMEFORMAT`, bash writes a `#<unix_seconds>` comment line
//! before each command. We support both forms.

use chrono::{Local, TimeZone};

use super::{HistoryEntry, Shell};

pub fn parse(contents: &str) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();
    let mut pending_ts = None;

    for line in contents.lines() {
        if let Some(ts) = parse_timestamp_line(line) {
            pending_ts = ts;
            continue;
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        entries.push(HistoryEntry {
            command: command.to_string(),
            timestamp: pending_ts.take(),
            shell: Shell::Bash,
        });
    }

    entries
}

/// Returns `Some(parsed)` if the line is a `#<digits>` timestamp comment.
/// The inner `Option` is the resolved time (None if out of range).
fn parse_timestamp_line(line: &str) -> Option<Option<chrono::DateTime<Local>>> {
    let rest = line.strip_prefix('#')?;
    let secs: i64 = rest.trim().parse().ok()?;
    Some(Local.timestamp_opt(secs, 0).single())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_commands() {
        let entries = parse("ls -la\ngit status\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "ls -la");
        assert!(entries[0].timestamp.is_none());
        assert_eq!(entries[0].shell, Shell::Bash);
    }

    #[test]
    fn parses_timestamped_commands() {
        let entries = parse("#1700000000\ngit status\n#1700000060\ncargo build\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "git status");
        assert!(entries[0].timestamp.is_some());
        assert_eq!(entries[1].command, "cargo build");
        assert!(entries[1].timestamp.is_some());
    }

    #[test]
    fn does_not_carry_timestamp_to_later_commands() {
        let entries = parse("#1700000000\ngit status\nls\n");
        assert!(entries[0].timestamp.is_some());
        assert!(entries[1].timestamp.is_none());
    }

    #[test]
    fn ignores_blank_lines() {
        assert_eq!(parse("ls\n\n\ncd\n").len(), 2);
    }
}
