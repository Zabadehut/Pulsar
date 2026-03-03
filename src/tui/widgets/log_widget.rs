use crate::collectors::{AlertLevel, LogEntry};
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct LogWidgetState<'a> {
    pub locale: Locale,
    pub targets: &'a [String],
    pub active_files: &'a [String],
    pub query: &'a str,
    pub input_active: bool,
    pub errors_only: bool,
    pub rotated_files: usize,
    pub entries: &'a [LogEntry],
    pub error: Option<&'a str>,
}

pub fn render(frame: &mut Frame, area: Rect, state: LogWidgetState<'_>, theme: &Theme) {
    let title = if state.input_active {
        text(state.locale, " ◉ LOGS LIVE / AJOUT ", " ◉ LIVE LOGS / ADD ")
    } else {
        text(state.locale, " ◉ LOGS LIVE ", " ◉ LIVE LOGS ")
    };

    let block = Block::default()
        .title(Line::from(vec![Span::styled(title, theme.title_style())]))
        .borders(Borders::ALL)
        .border_style(theme.border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", text(state.locale, "cibles", "targets")),
            theme.highlight_style(),
        ),
        Span::raw(if state.targets.is_empty() {
            text(state.locale, "aucune", "none")
        } else {
            ""
        }),
    ]));

    if state.targets.is_empty() {
        lines.push(Line::from(text(
            state.locale,
            "Appuyez sur l puis saisissez un chemin ou un motif, par ex. /var/log/app/*.log",
            "Press l then enter a path or pattern, for example /var/log/app/*.log",
        )));
    } else {
        for target in state.targets.iter().take(3) {
            lines.push(Line::from(target.as_str()));
        }
        if state.targets.len() > 3 {
            lines.push(Line::from(format!(
                "+{} {}",
                state.targets.len() - 3,
                text(state.locale, "autres", "more")
            )));
        }
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", text(state.locale, "mode", "mode")),
            theme.highlight_style(),
        ),
        Span::raw(if state.errors_only {
            text(state.locale, "erreurs", "errors")
        } else {
            text(state.locale, "tout", "all")
        }),
        Span::raw("  "),
        Span::styled(
            format!("{}: ", text(state.locale, "actifs", "active")),
            theme.highlight_style(),
        ),
        Span::raw(state.active_files.len().to_string()),
        Span::raw("  "),
        Span::styled(
            format!("{}: ", text(state.locale, "rotations", "rotations")),
            theme.highlight_style(),
        ),
        Span::raw(state.rotated_files.to_string()),
    ]));
    if !state.active_files.is_empty() {
        lines.push(Line::from(truncate(
            &state.active_files.join(", "),
            inner.width as usize,
        )));
    }

    lines.push(Line::from(vec![
        Span::styled(
            format!("{}: ", text(state.locale, "saisie", "input")),
            theme.highlight_style(),
        ),
        Span::raw(if state.query.is_empty() {
            text(state.locale, "(vide)", "(empty)")
        } else {
            state.query
        }),
    ]));

    if let Some(error) = state.error {
        lines.push(Line::from(vec![Span::styled(
            truncate(error, inner.width as usize),
            theme.alert_style(),
        )]));
    }

    lines.push(Line::default());

    if state.entries.is_empty() {
        lines.push(Line::from(text(
            state.locale,
            "Aucune entree recente issue du systeme ou des chemins surveilles.",
            "No recent entries from the system or watched paths.",
        )));
    } else {
        let available = inner.height.saturating_sub(lines.len() as u16) as usize;
        for entry in state.entries.iter().take(available) {
            let level_style = match entry.level {
                AlertLevel::Critical => theme.alert_style(),
                AlertLevel::Warning => theme.highlight_style(),
                AlertLevel::Info => theme.muted_style(),
            };
            let prefix = format!(
                "{} {} {}",
                level_label(&entry.level, state.locale),
                truncate(&entry.source, 10),
                truncate(&entry.origin, 22)
            );
            lines.push(Line::from(vec![
                Span::styled(prefix, level_style),
                Span::raw(" "),
                Span::raw(truncate(
                    &entry.message,
                    inner.width.saturating_sub(38) as usize,
                )),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .style(theme.muted_style())
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn level_label(level: &AlertLevel, locale: Locale) -> &'static str {
    match level {
        AlertLevel::Critical => text(locale, "ERR", "ERR"),
        AlertLevel::Warning => text(locale, "WRN", "WRN"),
        AlertLevel::Info => text(locale, "INF", "INF"),
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
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
