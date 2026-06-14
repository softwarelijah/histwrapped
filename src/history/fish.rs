//! Parser for fish history (`~/.local/share/fish/fish_history`).
//!
//! The format is YAML-ish, one entry per block:
//!
//! ```text
//! - cmd: git status
//!   when: 1700000000
//!   paths:
//!     - file.txt
//! ```
//!
//! fish escapes backslashes and newlines inside `cmd`. We hand-parse rather
//! than pull in a YAML crate, handling those two escapes and ignoring the
//! optional `paths` block.

use chrono::{Local, TimeZone};

use super::{HistoryEntry, Shell};

pub fn parse(contents: &str) -> Vec<HistoryEntry> {
    let mut entries = Vec::new();
    let mut current: Option<(String, Option<i64>)> = None;

    for line in contents.lines() {
        if let Some(cmd) = line.strip_prefix("- cmd: ") {
            if let Some((command, secs)) = current.take() {
                entries.push(build(command, secs));
            }
            current = Some((unescape(cmd), None));
        } else if let Some(when) = line.trim_start().strip_prefix("when: ") {
            if let Some((_, secs)) = current.as_mut() {
                *secs = when.trim().parse().ok();
            }
        }
        // Any other line (paths block, etc.) is ignored.
    }

    if let Some((command, secs)) = current.take() {
        entries.push(build(command, secs));
    }

    entries
}

fn build(command: String, secs: Option<i64>) -> HistoryEntry {
    HistoryEntry {
        command,
        timestamp: secs.and_then(|s| Local.timestamp_opt(s, 0).single()),
        shell: Shell::Fish,
    }
}

/// Reverse fish's escaping of `\\` and `\n` within a command.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entries_with_timestamps() {
        let input =
            "- cmd: git status\n  when: 1700000000\n- cmd: cargo build\n  when: 1700000060\n";
        let entries = parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "git status");
        assert!(entries[0].timestamp.is_some());
        assert_eq!(entries[1].command, "cargo build");
    }

    #[test]
    fn ignores_paths_block() {
        let input = "- cmd: vim file.txt\n  when: 1700000000\n  paths:\n    - file.txt\n- cmd: ls\n  when: 1700000005\n";
        let entries = parse(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].command, "vim file.txt");
        assert_eq!(entries[1].command, "ls");
    }

    #[test]
    fn unescapes_newlines_and_backslashes() {
        let entries = parse("- cmd: echo one\\ntwo\n  when: 1700000000\n");
        assert_eq!(entries[0].command, "echo one\ntwo");
    }
}
