//! Image export for the wrapped card.
//!
//! The SVG is built by hand (no dependencies) and renders anywhere, including
//! GitHub READMEs. PNG is produced by rasterizing that same SVG with resvg.

use std::path::Path;

use crate::analysis::Stats;

const W: u32 = 680;
const ROW_HEIGHT: i32 = 38;
const FIRST_ROW_Y: i32 = 160;
const FOOTER_GAP: i32 = 70;

/// Card height grows with the number of stat rows so it never looks empty.
fn height(row_count: usize) -> u32 {
    (FIRST_ROW_Y + row_count as i32 * ROW_HEIGHT + FOOTER_GAP) as u32
}

/// Build the wrapped card as an SVG document.
pub fn to_svg(stats: &Stats) -> String {
    let rows = rows(stats);
    let h = height(rows.len());
    let mut body = String::new();

    // Header.
    body.push_str(&text(
        W / 2,
        70,
        34,
        "700",
        "#ffffff",
        "middle",
        "HISTWRAPPED",
    ));
    body.push_str(&text(
        W / 2,
        100,
        15,
        "400",
        "#c9b8ff",
        "middle",
        "your command-line, wrapped",
    ));

    // Stat rows.
    let mut y = FIRST_ROW_Y;
    for (label, value) in &rows {
        body.push_str(&text(60, y, 18, "400", "#b8c0d8", "start", label));
        body.push_str(&text(W - 60, y, 18, "600", "#ffffff", "end", value));
        y += ROW_HEIGHT;
    }

    // Footer badge.
    body.push_str(&text(
        W / 2,
        h as i32 - 35,
        20,
        "700",
        "#7ee787",
        "middle",
        &format!("You are {}", stats.personality),
    ));

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{h}" viewBox="0 0 {W} {h}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#2a1a4a"/>
      <stop offset="100%" stop-color="#0d1b3a"/>
    </linearGradient>
  </defs>
  <rect width="{W}" height="{h}" rx="24" fill="url(#bg)"/>
  <rect x="6" y="6" width="{}" height="{}" rx="20" fill="none" stroke="#4a3a7a" stroke-width="2"/>
  <g font-family="monospace">
{body}  </g>
</svg>
"##,
        W - 12,
        h - 12,
    )
}

/// Write the card to a PNG file by rasterizing the SVG.
pub fn write_png(stats: &Stats, path: &Path) -> Result<(), String> {
    let svg = to_svg(stats);

    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();

    let tree = resvg::usvg::Tree::from_str(&svg, &options)
        .map_err(|e| format!("failed to build SVG tree: {e}"))?;

    let size = tree.size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(size.width().ceil() as u32, size.height().ceil() as u32)
            .ok_or_else(|| "failed to allocate pixmap".to_string())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap
        .save_png(path)
        .map_err(|e| format!("failed to write PNG: {e}"))
}

/// Write the card to an SVG file.
pub fn write_svg(stats: &Stats, path: &Path) -> Result<(), String> {
    std::fs::write(path, to_svg(stats)).map_err(|e| format!("failed to write SVG: {e}"))
}

/// The label/value rows shown on the card.
fn rows(stats: &Stats) -> Vec<(String, String)> {
    let mut rows = vec![
        ("Commands run".into(), group(stats.total_commands)),
        ("Unique commands".into(), group(stats.unique_commands)),
    ];
    if stats.active_days > 0 {
        rows.push(("Active days".into(), stats.active_days.to_string()));
        rows.push((
            "Longest streak".into(),
            format!("{} days", stats.longest_streak),
        ));
    }
    if let Some(hour) = stats.busiest_hour {
        rows.push(("Peak hour".into(), format!("{hour:02}:00")));
    }
    if let Some(top) = stats.top_programs.first() {
        rows.push((
            "Top tool".into(),
            format!("{} ({}x)", top.command, top.count),
        ));
    }
    if let Some(top) = stats.top_subcommands.first() {
        rows.push((
            "Top command".into(),
            format!("{} ({}x)", top.command, top.count),
        ));
    }
    rows
}

fn text(
    x: u32,
    y: i32,
    size: u32,
    weight: &str,
    fill: &str,
    anchor: &str,
    content: &str,
) -> String {
    format!(
        "    <text x=\"{x}\" y=\"{y}\" font-size=\"{size}\" font-weight=\"{weight}\" fill=\"{fill}\" text-anchor=\"{anchor}\">{}</text>\n",
        escape(content)
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn group(n: usize) -> String {
    let digits = n.to_string();
    let len = digits.len();
    let mut out = String::new();
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
    fn svg_is_well_formed_and_has_content() {
        let svg = to_svg(&sample());
        assert!(svg.starts_with("<svg"));
        assert!(svg.trim_end().ends_with("</svg>"));
        assert!(svg.contains("HISTWRAPPED"));
        assert!(svg.contains("4,012"));
        assert!(svg.contains("The Git Gardener"));
    }

    #[test]
    fn escapes_xml_special_chars() {
        let mut stats = sample();
        stats.top_subcommands = vec![CommandCount {
            command: "echo a > b & c".into(),
            count: 3,
        }];
        let svg = to_svg(&stats);
        assert!(svg.contains("&gt;"));
        assert!(svg.contains("&amp;"));
        assert!(!svg.contains("a > b"));
    }
}
