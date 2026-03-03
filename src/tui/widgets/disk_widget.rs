use crate::collectors::DiskMetrics;
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
    disks: &[DiskMetrics],
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ◉ DISK ",
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
        frame.render_widget(Paragraph::new("No disk data"), inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Device"),
        Cell::from("Used%"),
        Cell::from("R IOPS"),
        Cell::from("W IOPS"),
        Cell::from("Read KB/s"),
        Cell::from("Write KB/s"),
        Cell::from("Await"),
        Cell::from("Svc"),
        Cell::from("Qd"),
        Cell::from("Merge/s"),
        Cell::from("Util%"),
    ])
    .style(theme.highlight_style());

    let rows: Vec<Row> = disks
        .iter()
        .map(|d| {
            Row::new(vec![
                Cell::from(d.device.clone()),
                Cell::from(format!("{:.1}%", d.usage_pct)),
                Cell::from(format!("{}", d.read_iops)),
                Cell::from(format!("{}", d.write_iops)),
                Cell::from(format!("{}", d.read_throughput_kb)),
                Cell::from(format!("{}", d.write_throughput_kb)),
                Cell::from(format!("{:.1}ms", d.await_ms)),
                Cell::from(format!("{:.1}ms", d.service_time_ms)),
                Cell::from(format!("{:.2}", d.queue_depth)),
                Cell::from(format!(
                    "{}/{}",
                    d.read_merged_ops_sec, d.write_merged_ops_sec
                )),
                Cell::from(format!("{:.1}%", d.util_pct)),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(8),
        ratatui::layout::Constraint::Length(6),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(6),
    ];

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}
