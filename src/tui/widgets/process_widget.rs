use crate::collectors::ProcessMetrics;
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
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
    locale: Locale,
    detailed: bool,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(
                " ◉ {} (top {}) ",
                text(locale, "PROCESSUS", "PROCESSES"),
                processes.len()
            ),
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
        frame.render_widget(
            Paragraph::new(text(locale, "Pas de donnees processus", "No process data")),
            inner,
        );
        return;
    }

    let mut header_cells = vec![
        Cell::from("PID"),
        Cell::from(text(locale, "Nom", "Name")),
        Cell::from("CPU%"),
        Cell::from("MEM MB"),
        Cell::from(text(locale, "Etat", "State")),
        Cell::from("FDs"),
        Cell::from(text(locale, "User", "User")),
        Cell::from("JVM"),
    ];
    if detailed {
        header_cells.push(Cell::from("IO MB"));
    }
    let header = Row::new(header_cells).style(theme.highlight_style());

    let rows: Vec<Row> = processes
        .iter()
        .map(|p| {
            let cpu_style = if p.cpu_pct > 80.0 {
                theme.alert_style()
            } else {
                ratatui::style::Style::default().fg(theme.text)
            };

            let mut cells = vec![
                Cell::from(format!("{}", p.pid)),
                Cell::from(p.name.chars().take(16).collect::<String>()),
                Cell::from(format!("{:.1}", p.cpu_pct)).style(cpu_style),
                Cell::from(format!("{:.0}", p.mem_rss_kb as f64 / 1024.0)),
                Cell::from(format!("{:?}", p.state).chars().take(8).collect::<String>()),
                Cell::from(format!("{}", p.fd_count)),
                Cell::from(p.user.chars().take(10).collect::<String>()),
                Cell::from(if p.is_jvm { "JVM" } else { "" }),
            ];
            if detailed {
                cells.push(Cell::from(format!(
                    "{:.1}",
                    (p.io_read_bytes + p.io_write_bytes) as f64 / (1024.0 * 1024.0)
                )));
            }
            Row::new(cells)
        })
        .collect();

    let mut widths = vec![
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(17),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(6),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(4),
    ];
    if detailed {
        widths.push(ratatui::layout::Constraint::Length(7));
    }

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}
