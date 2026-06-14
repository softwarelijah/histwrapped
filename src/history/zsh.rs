//! Parser for zsh history (`~/.zsh_history`).
//!
//! With EXTENDED_HISTORY a line looks like `: 1700000000:0;git status`, where
//! the first number is the start time and the second is elapsed seconds.
//! Without it, a line is just the raw command. Multi-line commands use a
//! trailing backslash on each continued line.

use chrono::{Local, TimeZone};

use super::{HistoryEntry, Shell};

pub fn parse(contents: &str) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();
    let mut lines = contents.lines();

    while let Some(first) = lines.next() {
        let mut raw = first.to_string();
        while ends_with_continuation(&raw) {
            match lines.next() {
                Some(next) => {
                    raw.pop();
                    raw.push('\n');
                    raw.push_str(next);
                }
                None => break,
            }
        }

        if let Some(entry) = parse_entry(&raw) {
            entries.push(entry);
        }
    }

    entries
}

/// A line continues if it ends with an odd number of backslashes.
fn ends_with_continuation(line: &str) -> bool {
    line.chars().rev().take_while(|&c| c == '\\').count() % 2 == 1
}

fn parse_entry(raw: &str) -> Option<HistoryEntry> {
    let raw = raw.trim_end_matches('\n');

    // Extended form: ": <start>:<elapsed>;<command>"
    if let Some(rest) = raw.strip_prefix(": ") {
        if let Some((meta, command)) = rest.split_once(';') {
            let timestamp = meta
                .split_once(':')
                .and_then(|(start, _elapsed)| start.trim().parse::<i64>().ok())
                .and_then(|secs| Local.timestamp_opt(secs, 0).single());

            let command = command.trim().to_string();
            if command.is_empty() {
                return None;
            }
            return Some(HistoryEntry {
                command,
                timestamp,
                shell: Shell::Zsh,
            });
        }
    }

    let command = raw.trim().to_string();
    if command.is_empty() {
        return None;
    }
    Some(HistoryEntry {
        command,
        timestamp: None,
        shell: Shell::Zsh,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_extended_history() {
        let input = ": 1700000000:0;git status -s\n: 1700000005:2;cargo build\n";
        let entries = parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "git status -s");
        assert_eq!(entries[0].program(), "git");
        assert!(entries[0].timestamp.is_some());
        assert_eq!(entries[1].command, "cargo build");
    }

    #[test]
    fn parses_plain_history() {
        let entries = parse("ls -la\ncd ..\n");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "ls -la");
        assert!(entries[0].timestamp.is_none());
    }

    #[test]
    fn joins_multiline_commands() {
        let entries = parse(": 1700000000:0;echo one \\\ntwo three\n");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "echo one \ntwo three");
    }

    #[test]
    fn skips_blank_lines() {
        assert_eq!(parse("ls\n\n\ncd\n").len(), 2);
    }

    #[test]
    fn keeps_semicolons_in_command() {
        let entries = parse(": 1700000000:0;echo hi; echo bye\n");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "echo hi; echo bye");
    }
}
