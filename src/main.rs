//! histwrapped: Spotify Wrapped for your terminal.
//!
//! Loads shell history, computes stats, and renders them as a text summary
//! (`stats`), JSON (`export`), or a shareable card (`wrapped`).

mod analysis;
mod history;
mod wrapped;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use history::{detect, HistoryEntry, Shell};

#[derive(Parser)]
#[command(
    name = "histwrapped",
    version,
    about = "Spotify Wrapped for your terminal — see your command-line year in one screenshot."
)]
struct Cli {
    /// Path to a history file (overrides shell auto-detection).
    #[arg(long, global = true, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Force a specific shell parser instead of auto-detecting.
    #[arg(long, global = true, value_enum)]
    shell: Option<ShellArg>,

    /// How many entries to show in "top" lists.
    #[arg(long, global = true, default_value_t = 10)]
    top: usize,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print a quick summary of your command-line stats.
    Stats,
    /// Render a shareable "wrapped" card (coming soon).
    Wrapped,
    /// Dump computed stats as JSON.
    Export,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ShellArg {
    Zsh,
    Bash,
    Fish,
}

impl From<ShellArg> for Shell {
    fn from(value: ShellArg) -> Self {
        match value {
            ShellArg::Zsh => Shell::Zsh,
            ShellArg::Bash => Shell::Bash,
            ShellArg::Fish => Shell::Fish,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    let entries = load_entries(cli)?;
    if entries.is_empty() {
        return Err("no history entries found — is your history file empty?".to_string());
    }

    let stats = analysis::analyze(&entries, cli.top);

    match cli.command {
        Command::Stats => print_stats(&stats),
        Command::Export => {
            let json = serde_json::to_string_pretty(&stats)
                .map_err(|e| format!("failed to serialize stats: {e}"))?;
            println!("{json}");
        }
        Command::Wrapped => {
            println!("{}", wrapped::render(&stats));
        }
    }

    Ok(())
}

/// Resolve the shell + path, read the file, and parse it into entries.
fn load_entries(cli: &Cli) -> Result<Vec<HistoryEntry>, String> {
    let shell = cli
        .shell
        .map(Shell::from)
        .unwrap_or_else(detect::detect_shell);

    let path = match &cli.file {
        Some(p) => p.clone(),
        None => detect::history_path(shell)
            .ok_or_else(|| "could not determine a history-file path".to_string())?,
    };

    let contents =
        std::fs::read(&path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let contents = String::from_utf8_lossy(&contents);

    let entries = match shell {
        Shell::Zsh => history::zsh::parse(&contents),
        Shell::Bash => history::bash::parse(&contents),
        Shell::Fish => history::fish::parse(&contents),
    };

    Ok(entries)
}

fn print_stats(stats: &analysis::Stats) {
    println!("histwrapped");
    println!();
    println!("  total commands : {}", stats.total_commands);
    println!("  unique commands: {}", stats.unique_commands);
    if stats.active_days > 0 {
        println!("  active days    : {}", stats.active_days);
        println!("  longest streak : {} days", stats.longest_streak);
    }
    println!("  personality    : {}", stats.personality);
    println!();

    print_top("Top programs", &stats.top_programs, 20);
    println!();
    print_top("Top subcommands", &stats.top_subcommands, 30);
    println!();
    print_top("Top commands", &stats.top_commands, 30);

    if stats.timestamped > 0 {
        println!();
        println!("  Activity by hour:");
        print_hour_histogram(&stats.hour_histogram);
    }
}

fn print_top(title: &str, items: &[analysis::CommandCount], width: usize) {
    if items.is_empty() {
        return;
    }
    println!("  {title}:");
    for (i, c) in items.iter().enumerate() {
        println!("    {:>2}. {:<width$} {}", i + 1, c.command, c.count);
    }
}

/// A compact bar chart of the 24-hour histogram using block characters.
fn print_hour_histogram(hist: &[usize; 24]) {
    let max = hist.iter().copied().max().unwrap_or(0).max(1);
    for (hour, &count) in hist.iter().enumerate() {
        let width = (count * 30) / max;
        let bar = "█".repeat(width);
        println!("    {hour:02}:00 {bar} {count}");
    }
}
