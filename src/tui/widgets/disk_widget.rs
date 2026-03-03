use crate::collectors::DiskMetrics;
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
    disks: &[DiskMetrics],
    locale: Locale,
    detailed: bool,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            text(locale, " ◉ DISQUE ", " ◉ DISK "),
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

    if disks.is_empty() {
        frame.render_widget(
            Paragraph::new(text(locale, "Pas de donnees disque", "No disk data")),
            inner,
        );
        return;
    }

    let mut header_cells = vec![
        Cell::from(text(locale, "Device", "Device")),
        Cell::from(text(locale, "Struct", "Struct")),
        Cell::from(text(locale, "Proto", "Proto")),
        Cell::from(text(locale, "Monte", "Mount")),
        Cell::from(text(locale, "Use%", "Used%")),
        Cell::from("R IOPS"),
        Cell::from("W IOPS"),
        Cell::from(text(locale, "Lect KB/s", "Read KB/s")),
        Cell::from(text(locale, "Ecr KB/s", "Write KB/s")),
        Cell::from("Await"),
        Cell::from("Svc"),
        Cell::from("Qd"),
        Cell::from("Util%"),
    ];
    if detailed {
        header_cells.push(Cell::from(text(locale, "FS", "FS")));
        header_cells.push(Cell::from(text(locale, "Parent", "Parent")));
    }
    let header = Row::new(header_cells).style(theme.highlight_style());

    let rows: Vec<Row> = disks
        .iter()
        .map(|d| {
            let mut cells = vec![
                Cell::from(d.device.clone()),
                Cell::from(d.structure_hint.chars().take(10).collect::<String>()),
                Cell::from(d.protocol_hint.chars().take(10).collect::<String>()),
                Cell::from(d.mount_point.chars().take(10).collect::<String>()),
                Cell::from(format!("{:.1}%", d.usage_pct)),
                Cell::from(format!("{}", d.read_iops)),
                Cell::from(format!("{}", d.write_iops)),
                Cell::from(format!("{}", d.read_throughput_kb)),
                Cell::from(format!("{}", d.write_throughput_kb)),
                Cell::from(format!("{:.1}ms", d.await_ms)),
                Cell::from(format!("{:.1}ms", d.service_time_ms)),
                Cell::from(format!("{:.2}", d.queue_depth)),
                Cell::from(format!("{:.1}%", d.util_pct)),
            ];
            if detailed {
                cells.push(Cell::from(d.filesystem.chars().take(8).collect::<String>()));
                cells.push(Cell::from(d.parent.chars().take(10).collect::<String>()));
            }
            Row::new(cells)
        })
        .collect();

    let mut widths = vec![
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(6),
        ratatui::layout::Constraint::Length(6),
    ];
    if detailed {
        widths.push(ratatui::layout::Constraint::Length(9));
        widths.push(ratatui::layout::Constraint::Length(11));
    }

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}
