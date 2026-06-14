//! Interactive explorer built on ratatui: a one-screen dashboard of the same
//! stats, with the hour histogram drawn as a bar chart. Press q or Esc to exit.

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::analysis::{CommandCount, Stats};

/// Run the TUI until the user quits. Restores the terminal on the way out.
pub fn run(stats: &Stats) -> Result<(), String> {
    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, stats);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut ratatui::DefaultTerminal, stats: &Stats) -> Result<(), String> {
    loop {
        terminal
            .draw(|frame| draw(frame, stats))
            .map_err(|e| format!("draw failed: {e}"))?;

        if let Event::Key(key) = event::read().map_err(|e| format!("input failed: {e}"))? {
            if key.kind == KeyEventKind::Press
                && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
            {
                return Ok(());
            }
        }
    }
}

fn draw(frame: &mut Frame, stats: &Stats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(6),
            Constraint::Length(12),
        ])
        .split(frame.area());

    frame.render_widget(summary(stats), chunks[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    frame.render_widget(top_list("Top programs", &stats.top_programs), columns[0]);
    frame.render_widget(
        top_list("Top subcommands", &stats.top_subcommands),
        columns[1],
    );

    frame.render_widget(hour_chart(stats), chunks[2]);
}

fn summary(stats: &Stats) -> Paragraph<'static> {
    let mut lines = vec![
        kv("Commands run", stats.total_commands.to_string()),
        kv("Unique commands", stats.unique_commands.to_string()),
    ];
    if stats.active_days > 0 {
        lines.push(kv("Active days", stats.active_days.to_string()));
        lines.push(kv(
            "Longest streak",
            format!("{} days", stats.longest_streak),
        ));
    }
    lines.push(Line::from(vec![
        Span::styled("You are ", Style::default().fg(Color::Gray)),
        Span::styled(
            stats.personality.clone(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" histwrapped (press q to quit) "),
    )
}

fn kv(label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<16}"), Style::default().fg(Color::Gray)),
        Span::styled(value, Style::default().add_modifier(Modifier::BOLD)),
    ])
}

fn top_list(title: &'static str, items: &[CommandCount]) -> List<'static> {
    let rows: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, c)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>2}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(format!("{:<24}", c.command)),
                Span::styled(c.count.to_string(), Style::default().fg(Color::Cyan)),
            ]))
        })
        .collect();

    List::new(rows).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {title} ")),
    )
}

fn hour_chart(stats: &Stats) -> BarChart<'static> {
    let bars: Vec<Bar> = stats
        .hour_histogram
        .iter()
        .enumerate()
        .map(|(hour, &count)| {
            Bar::default()
                .value(count as u64)
                .label(Line::from(format!("{hour:02}")))
                .style(Style::default().fg(Color::Magenta))
        })
        .collect();

    let title = match stats.busiest_hour {
        Some(_) if stats.timestamped > 0 => " Activity by hour ".to_string(),
        _ => " Activity by hour (no timestamps in history) ".to_string(),
    };

    BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .data(BarGroup::default().bars(&bars))
        .bar_width(2)
        .bar_gap(1)
}
