use crate::collectors::SystemMetrics;
use crate::tui::theme::Theme;
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
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ◉ SYSTEM ",
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
        frame.render_widget(Paragraph::new("Collecting..."), inner);
        return;
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        paragraph(format!(" host: {}", system.hostname), theme),
        rows[0],
    );
    frame.render_widget(
        paragraph(
            format!(
                " os: {} {}  kernel: {}",
                system.os_name, system.os_version, system.kernel_version
            ),
            theme,
        ),
        rows[1],
    );
    frame.render_widget(
        paragraph(
            format!(" arch: {}  cpus: {}", system.architecture, system.cpu_count),
            theme,
        ),
        rows[2],
    );
    frame.render_widget(
        paragraph(format!(" uptime: {}s", system.uptime_seconds), theme),
        rows[3],
    );
}

fn paragraph(text: String, theme: &Theme) -> Paragraph<'static> {
    Paragraph::new(text).style(ratatui::style::Style::default().fg(theme.neutral))
}
