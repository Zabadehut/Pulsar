use crate::collectors::SystemMetrics;
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    metrics: Option<&SystemMetrics>,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            text(locale, " ◉ SYSTEME ", " ◉ SYSTEM "),
            if highlighted {
                theme.highlight_style()
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

    let Some(system) = metrics else {
        frame.render_widget(
            Paragraph::new(text(locale, "Collecte...", "Collecting...")),
            inner,
        );
        return;
    };

    let mut constraints = vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ];
    if detailed && inner.height >= 5 {
        constraints.push(Constraint::Length(1));
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    frame.render_widget(
        paragraph(
            format!(" {}: {}", text(locale, "hote", "host"), system.hostname),
            theme,
        ),
        rows[0],
    );
    frame.render_widget(
        paragraph(
            format!(
                " os: {} {}  {}: {}",
                system.os_name,
                system.os_version,
                text(locale, "noyau", "kernel"),
                system.kernel_version
            ),
            theme,
        ),
        rows[1],
    );
    frame.render_widget(
        paragraph(
            format!(
                " arch: {}  {}: {}",
                system.architecture,
                text(locale, "cpus", "cpus"),
                system.cpu_count
            ),
            theme,
        ),
        rows[2],
    );
    frame.render_widget(
        paragraph(
            format!(
                " {}: {}s",
                text(locale, "uptime", "uptime"),
                system.uptime_seconds
            ),
            theme,
        ),
        rows[3],
    );

    if detailed && rows.len() > 4 {
        frame.render_widget(
            paragraph(
                format!(
                    " {}: {}",
                    text(locale, "version", "version"),
                    system.os_version
                ),
                theme,
            ),
            rows[4],
        );
    }
}

fn paragraph(text: String, theme: &Theme) -> Paragraph<'static> {
    Paragraph::new(text).style(ratatui::style::Style::default().fg(theme.neutral))
}
