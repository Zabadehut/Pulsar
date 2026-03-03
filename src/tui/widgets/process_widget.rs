use crate::collectors::ProcessMetrics;
use crate::tui::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    processes: &[ProcessMetrics],
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(" ◉ PROCESSES (top {}) ", processes.len()),
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

    if processes.is_empty() {
        frame.render_widget(Paragraph::new("No process data"), inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from("PID"),
        Cell::from("Name"),
        Cell::from("CPU%"),
        Cell::from("MEM MB"),
        Cell::from("State"),
        Cell::from("FDs"),
        Cell::from("User"),
        Cell::from("JVM"),
    ])
    .style(theme.highlight_style());

    let rows: Vec<Row> = processes
        .iter()
        .map(|p| {
            let cpu_style = if p.cpu_pct > 80.0 {
                theme.alert_style()
            } else {
                ratatui::style::Style::default().fg(theme.text)
            };

            Row::new(vec![
                Cell::from(format!("{}", p.pid)),
                Cell::from(p.name.chars().take(16).collect::<String>()),
                Cell::from(format!("{:.1}", p.cpu_pct)).style(cpu_style),
                Cell::from(format!("{:.0}", p.mem_rss_kb as f64 / 1024.0)),
                Cell::from(format!("{:?}", p.state).chars().take(8).collect::<String>()),
                Cell::from(format!("{}", p.fd_count)),
                Cell::from(p.user.chars().take(10).collect::<String>()),
                Cell::from(if p.is_jvm { "JVM" } else { "" }),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(17),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(6),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(4),
    ];

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}
