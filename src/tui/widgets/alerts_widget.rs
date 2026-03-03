use crate::collectors::{Alert as PulsarAlert, AlertLevel};
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
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
    locale: Locale,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(
                " ◉ {} ({}) ",
                text(locale, "ALERTES", "ALERTS"),
                alerts.len()
            ),
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
            Paragraph::new(text(locale, "Aucune alerte active", "No active alerts"))
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
                Span::styled(level_label(&alert.level, locale), style),
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

fn level_label(level: &AlertLevel, locale: Locale) -> &'static str {
    match level {
        AlertLevel::Critical => text(locale, "CRIT", "CRIT"),
        AlertLevel::Warning => text(locale, "ALRT", "WARN"),
        AlertLevel::Info => text(locale, "INFO", "INFO"),
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
