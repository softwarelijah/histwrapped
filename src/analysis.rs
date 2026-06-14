//! Statistics computed from parsed history.
//!
//! Everything here is a pure function over `&[HistoryEntry]`: no I/O and no
//! shell-specific logic, so the TUI and the `wrapped` card render from the
//! exact same numbers.

use std::collections::HashMap;

use chrono::Timelike;
use serde::Serialize;

use crate::history::HistoryEntry;

/// A command and how many times it appeared.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandCount {
    pub command: String,
    pub count: usize,
}

/// The full set of stats the rest of the app renders.
#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    /// Total commands parsed (including repeats).
    pub total_commands: usize,
    /// Number of distinct full command lines.
    pub unique_commands: usize,
    /// Top programs by invocation count (e.g. `git`, `cargo`, `ls`).
    pub top_programs: Vec<CommandCount>,
    /// Top full command lines (e.g. `git status -s`).
    pub top_commands: Vec<CommandCount>,
    /// Count of commands run in each hour of the day, index 0..=23.
    /// All zero if no entries carried timestamps.
    pub hour_histogram: [usize; 24],
    /// How many entries had a usable timestamp.
    pub timestamped: usize,
}

/// Compute every statistic from a slice of entries.
///
/// `top_n` bounds the length of the `top_programs` / `top_commands` lists.
pub fn analyze(entries: &[HistoryEntry], top_n: usize) -> Stats {
    let mut program_counts: HashMap<&str, usize> = HashMap::new();
    let mut command_counts: HashMap<&str, usize> = HashMap::new();
    let mut hour_histogram = [0usize; 24];
    let mut timestamped = 0;

    for entry in entries {
        *program_counts.entry(entry.program()).or_insert(0) += 1;
        *command_counts.entry(entry.command.as_str()).or_insert(0) += 1;

        if let Some(ts) = entry.timestamp {
            hour_histogram[ts.hour() as usize] += 1;
            timestamped += 1;
        }
    }

    Stats {
        total_commands: entries.len(),
        unique_commands: command_counts.len(),
        top_programs: top_counts(&program_counts, top_n),
        top_commands: top_counts(&command_counts, top_n),
        hour_histogram,
        timestamped,
    }
}

/// Sort a count map descending by count (ties broken alphabetically for stable
/// output) and take the top `n`.
fn top_counts(counts: &HashMap<&str, usize>, n: usize) -> Vec<CommandCount> {
    let mut pairs: Vec<(&str, usize)> = counts.iter().map(|(&k, &v)| (k, v)).collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    pairs
        .into_iter()
        .take(n)
        .map(|(command, count)| CommandCount {
            command: command.to_string(),
            count,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::Shell;
    use chrono::{Local, TimeZone};

    fn entry(command: &str, hour: Option<u32>) -> HistoryEntry {
        let timestamp = hour.map(|h| {
            Local
                .with_ymd_and_hms(2026, 1, 1, h, 0, 0)
                .single()
                .unwrap()
        });
        HistoryEntry {
            command: command.to_string(),
            timestamp,
            shell: Shell::Zsh,
        }
    }

    #[test]
    fn counts_totals_and_uniques() {
        let entries = vec![
            entry("git status", None),
            entry("git status", None),
            entry("ls", None),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.unique_commands, 2);
    }

    #[test]
    fn ranks_top_programs() {
        let entries = vec![
            entry("git status", None),
            entry("git commit", None),
            entry("ls -la", None),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.top_programs[0].command, "git");
        assert_eq!(stats.top_programs[0].count, 2);
    }

    #[test]
    fn builds_hour_histogram() {
        let entries = vec![
            entry("a", Some(9)),
            entry("b", Some(9)),
            entry("c", Some(14)),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.hour_histogram[9], 2);
        assert_eq!(stats.hour_histogram[14], 1);
        assert_eq!(stats.timestamped, 3);
    }

    #[test]
    fn respects_top_n() {
        let entries = vec![entry("a", None), entry("b", None), entry("c", None)];
        let stats = analyze(&entries, 2);
        assert_eq!(stats.top_programs.len(), 2);
    }
}
