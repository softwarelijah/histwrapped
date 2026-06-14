//! Renders the shareable "wrapped" card: a single boxed summary built for
//! screenshots. Pure string building so it can be tested and later reused by
//! an image exporter.

use crate::analysis::Stats;

const WIDTH: usize = 50;

/// Build the full card as a multi-line string.
pub fn render(stats: &Stats) -> String {
    let mut lines: Vec<String> = vec![
        center("HISTWRAPPED"),
        center("your command-line, wrapped"),
        String::new(),
    ];

    lines.push(field("Commands run", &group(stats.total_commands)));
    lines.push(field("Unique commands", &group(stats.unique_commands)));
    if stats.active_days > 0 {
        lines.push(field("Active days", &stats.active_days.to_string()));
        lines.push(field(
            "Longest streak",
            &format!("{} days", stats.longest_streak),
        ));
    }
    if let Some(hour) = stats.busiest_hour {
        lines.push(field("Peak hour", &format!("{hour:02}:00")));
    }
    lines.push(String::new());

    if let Some(top) = stats.top_programs.first() {
        lines.push(field(
            "Top tool",
            &format!("{} ({}x)", top.command, top.count),
        ));
    }
    if let Some(top) = stats.top_subcommands.first() {
        lines.push(field(
            "Top command",
            &format!("{} ({}x)", top.command, top.count),
        ));
    }
    lines.push(String::new());

    lines.push(center(&format!("You are {}", stats.personality)));

    boxed(&lines)
}

/// Wrap content lines in a rounded border of `WIDTH` interior columns.
fn boxed(lines: &[String]) -> String {
    let mut out = String::new();
    out.push('╭');
    out.push_str(&"─".repeat(WIDTH));
    out.push_str("╮\n");

    for line in lines {
        out.push_str(&format!("│{}│\n", pad(line)));
    }

    out.push('╰');
    out.push_str(&"─".repeat(WIDTH));
    out.push('╯');
    out
}

/// A "Label .......... value" row.
fn field(label: &str, value: &str) -> String {
    let used = display_width(label) + display_width(value);
    let dots = WIDTH.saturating_sub(used + 4);
    format!("  {label} {} {value}  ", ".".repeat(dots))
}

/// Center text within the interior width.
fn center(text: &str) -> String {
    let w = display_width(text);
    if w >= WIDTH {
        return text.to_string();
    }
    let left = (WIDTH - w) / 2;
    let right = WIDTH - w - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

/// Right-pad (or truncate) a line to exactly the interior width.
fn pad(line: &str) -> String {
    let w = display_width(line);
    if w >= WIDTH {
        line.chars().take(WIDTH).collect()
    } else {
        format!("{}{}", line, " ".repeat(WIDTH - w))
    }
}

/// Approximate display width: counts chars (the card uses plain ASCII content).
fn display_width(s: &str) -> usize {
    s.chars().count()
}

/// Group a number with thousands separators, e.g. 4012 -> "4,012".
fn group(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::CommandCount;

    fn sample() -> Stats {
        Stats {
            total_commands: 4012,
            unique_commands: 312,
            top_programs: vec![CommandCount {
                command: "git".into(),
                count: 1600,
            }],
            top_commands: vec![],
            top_subcommands: vec![CommandCount {
                command: "git status".into(),
                count: 800,
            }],
            hour_histogram: [0; 24],
            busiest_hour: Some(14),
            active_days: 120,
            longest_streak: 9,
            first_seen: None,
            last_seen: None,
            timestamped: 4012,
            personality: "The Git Gardener".into(),
        }
    }

    #[test]
    fn groups_thousands() {
        assert_eq!(group(4012), "4,012");
        assert_eq!(group(999), "999");
        assert_eq!(group(1000000), "1,000,000");
    }

    #[test]
    fn every_row_is_the_same_width() {
        let card = render(&sample());
        let widths: Vec<usize> = card.lines().map(|l| l.chars().count()).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "rows misaligned: {widths:?}"
        );
    }

    #[test]
    fn includes_key_facts() {
        let card = render(&sample());
        assert!(card.contains("4,012"));
        assert!(card.contains("The Git Gardener"));
        assert!(card.contains("git status"));
    }
}
