//! Statistics computed from parsed history.
//!
//! Everything here is a pure function over `&[HistoryEntry]`: no I/O and no
//! shell-specific logic, so the TUI and the `wrapped` card render from the
//! exact same numbers.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local, NaiveDate, Timelike};
use serde::Serialize;

use crate::history::HistoryEntry;

/// Programs whose second token is meaningful enough to track on its own,
/// e.g. `git status`, `cargo build`, `docker run`.
const MULTI_TOOLS: &[&str] = &[
    "git",
    "cargo",
    "docker",
    "npm",
    "yarn",
    "pnpm",
    "kubectl",
    "brew",
    "go",
    "pip",
    "pip3",
    "apt",
    "systemctl",
    "make",
    "terraform",
    "gh",
];

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
    /// Top two-token subcommands for known multi-tools (e.g. `git status`).
    pub top_subcommands: Vec<CommandCount>,
    /// Count of commands run in each hour of the day, index 0..=23.
    pub hour_histogram: [usize; 24],
    /// Hour with the most activity, if any entries were timestamped.
    pub busiest_hour: Option<usize>,
    /// Distinct calendar days with at least one command.
    pub active_days: usize,
    /// Longest run of consecutive active days.
    pub longest_streak: usize,
    /// Earliest and latest timestamps seen.
    pub first_seen: Option<DateTime<Local>>,
    pub last_seen: Option<DateTime<Local>>,
    /// How many entries had a usable timestamp.
    pub timestamped: usize,
    /// A playful archetype derived from the top program.
    pub personality: String,
}

/// Compute every statistic from a slice of entries.
///
/// `top_n` bounds the length of the various "top" lists.
pub fn analyze(entries: &[HistoryEntry], top_n: usize) -> Stats {
    let mut program_counts: HashMap<&str, usize> = HashMap::new();
    let mut command_counts: HashMap<&str, usize> = HashMap::new();
    let mut subcommand_counts: HashMap<String, usize> = HashMap::new();
    let mut hour_histogram = [0usize; 24];
    let mut days: HashSet<NaiveDate> = HashSet::new();
    let mut first_seen: Option<DateTime<Local>> = None;
    let mut last_seen: Option<DateTime<Local>> = None;
    let mut timestamped = 0;

    for entry in entries {
        let program = entry.program();
        *program_counts.entry(program).or_insert(0) += 1;
        *command_counts.entry(entry.command.as_str()).or_insert(0) += 1;

        if MULTI_TOOLS.contains(&program) {
            if let Some(sub) = entry.command.split_whitespace().nth(1) {
                *subcommand_counts
                    .entry(format!("{program} {sub}"))
                    .or_insert(0) += 1;
            }
        }

        if let Some(ts) = entry.timestamp {
            hour_histogram[ts.hour() as usize] += 1;
            days.insert(ts.date_naive());
            timestamped += 1;
            first_seen = Some(first_seen.map_or(ts, |f| f.min(ts)));
            last_seen = Some(last_seen.map_or(ts, |l| l.max(ts)));
        }
    }

    let busiest_hour = (timestamped > 0).then(|| {
        hour_histogram
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(hour, _)| hour)
            .unwrap_or(0)
    });

    let personality = personality_for(&program_counts);

    Stats {
        total_commands: entries.len(),
        unique_commands: command_counts.len(),
        top_programs: top_counts_str(&program_counts, top_n),
        top_commands: top_counts_str(&command_counts, top_n),
        top_subcommands: top_counts_owned(&subcommand_counts, top_n),
        hour_histogram,
        busiest_hour,
        active_days: days.len(),
        longest_streak: longest_streak(&days),
        first_seen,
        last_seen,
        timestamped,
        personality,
    }
}

fn top_counts_str(counts: &HashMap<&str, usize>, n: usize) -> Vec<CommandCount> {
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

fn top_counts_owned(counts: &HashMap<String, usize>, n: usize) -> Vec<CommandCount> {
    let mut pairs: Vec<(&String, usize)> = counts.iter().map(|(k, &v)| (k, v)).collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    pairs
        .into_iter()
        .take(n)
        .map(|(command, count)| CommandCount {
            command: command.clone(),
            count,
        })
        .collect()
}

/// Longest run of consecutive calendar days in the active-day set.
fn longest_streak(days: &HashSet<NaiveDate>) -> usize {
    if days.is_empty() {
        return 0;
    }
    let mut sorted: Vec<NaiveDate> = days.iter().copied().collect();
    sorted.sort();

    let mut best = 1;
    let mut current = 1;
    for pair in sorted.windows(2) {
        if pair[1] == pair[0].succ_opt().unwrap_or(pair[0]) {
            current += 1;
            best = best.max(current);
        } else {
            current = 1;
        }
    }
    best
}

/// Pick a playful archetype based on the most-used program.
fn personality_for(program_counts: &HashMap<&str, usize>) -> String {
    let top = program_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&program, _)| program)
        .unwrap_or("");

    let label = match top {
        "git" | "gh" => "The Git Gardener",
        "docker" | "kubectl" | "podman" => "The Container Captain",
        "cargo" | "rustc" => "The Rustacean",
        "npm" | "yarn" | "pnpm" | "node" => "The Node Wrangler",
        "vim" | "nvim" | "nano" | "emacs" => "The Modal Monk",
        "cd" | "ls" | "clear" => "The Terminal Pacer",
        "python" | "python3" | "pip" | "pip3" => "The Snake Charmer",
        "" => "The Quiet One",
        _ => "The Shell Tinkerer",
    };
    label.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::Shell;
    use chrono::TimeZone;

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

    fn dated(command: &str, y: i32, m: u32, d: u32) -> HistoryEntry {
        HistoryEntry {
            command: command.to_string(),
            timestamp: Local.with_ymd_and_hms(y, m, d, 12, 0, 0).single(),
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
    fn tracks_subcommands_for_multi_tools() {
        let entries = vec![
            entry("git status", None),
            entry("git status", None),
            entry("git push", None),
            entry("ls -la", None),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.top_subcommands[0].command, "git status");
        assert_eq!(stats.top_subcommands[0].count, 2);
        assert!(stats.top_subcommands.iter().all(|c| c.command != "ls -la"));
    }

    #[test]
    fn builds_hour_histogram_and_busiest_hour() {
        let entries = vec![
            entry("a", Some(9)),
            entry("b", Some(9)),
            entry("c", Some(14)),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.hour_histogram[9], 2);
        assert_eq!(stats.busiest_hour, Some(9));
        assert_eq!(stats.timestamped, 3);
    }

    #[test]
    fn computes_active_days_and_streak() {
        let entries = vec![
            dated("a", 2026, 1, 1),
            dated("b", 2026, 1, 2),
            dated("c", 2026, 1, 3),
            dated("d", 2026, 1, 10),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.active_days, 4);
        assert_eq!(stats.longest_streak, 3);
    }

    #[test]
    fn assigns_personality_from_top_program() {
        let entries = vec![
            entry("git status", None),
            entry("git push", None),
            entry("ls", None),
        ];
        let stats = analyze(&entries, 10);
        assert_eq!(stats.personality, "The Git Gardener");
    }

    #[test]
    fn respects_top_n() {
        let entries = vec![entry("a", None), entry("b", None), entry("c", None)];
        let stats = analyze(&entries, 2);
        assert_eq!(stats.top_programs.len(), 2);
    }
}
