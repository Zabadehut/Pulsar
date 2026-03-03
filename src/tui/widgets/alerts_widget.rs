use crate::collectors::{Alert as PulsarAlert, AlertLevel};
use crate::tui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    alerts: &[PulsarAlert],
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(" ◉ ALERTS ({}) ", alerts.len()),
            if highlighted {
                theme.alert_style()
            } else {
                theme.title_style()
            },
        )]))
        .borders(Borders::ALL)
        .border_style(if highlighted {
            theme.highlight_style()
        } else {
            theme.border_style()
        });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if alerts.is_empty() {
        frame.render_widget(
            Paragraph::new("No active alerts")
                .style(ratatui::style::Style::default().fg(theme.neutral)),
            inner,
        );
        return;
    }

    let lines: Vec<Line> = alerts
        .iter()
        .take(inner.height as usize)
        .map(|alert| {
            let style = match alert.level {
                AlertLevel::Critical | AlertLevel::Warning => theme.alert_style(),
                AlertLevel::Info => theme.highlight_style(),
            };
            Line::from(vec![
                Span::styled(level_label(&alert.level), style),
                Span::raw(" "),
                Span::raw(truncate_text(
                    &alert.message,
                    inner.width.saturating_sub(5) as usize,
                )),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
}

fn level_label(level: &AlertLevel) -> &'static str {
    match level {
        AlertLevel::Critical => "CRIT",
        AlertLevel::Warning => "WARN",
        AlertLevel::Info => "INFO",
    }
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        value
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>()
            + "…"
    }
}
