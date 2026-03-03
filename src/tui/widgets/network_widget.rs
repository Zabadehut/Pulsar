use crate::collectors::NetworkMetrics;
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
    networks: &[NetworkMetrics],
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ◉ NETWORK ",
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

    if networks.is_empty() {
        frame.render_widget(Paragraph::new("No network data"), inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Interface"),
        Cell::from("RX KB/s"),
        Cell::from("TX KB/s"),
        Cell::from("RX pkt/s"),
        Cell::from("TX pkt/s"),
        Cell::from("Errors"),
        Cell::from("Drops"),
        Cell::from("TCP"),
        Cell::from("UDP/Rtx"),
    ])
    .style(theme.highlight_style());

    let rows: Vec<Row> = networks
        .iter()
        .map(|n| {
            Row::new(vec![
                Cell::from(n.interface.clone()),
                Cell::from(format!("{}", n.rx_bytes_sec / 1024)),
                Cell::from(format!("{}", n.tx_bytes_sec / 1024)),
                Cell::from(format!("{}", n.rx_packets_sec)),
                Cell::from(format!("{}", n.tx_packets_sec)),
                Cell::from(format!("{}", n.rx_errors + n.tx_errors)),
                Cell::from(format!("{}", n.rx_dropped + n.tx_dropped)),
                Cell::from(format!(
                    "{}/{}/{}",
                    n.connections_established, n.tcp_listen, n.tcp_time_wait
                )),
                Cell::from(format!("{}/{}", n.udp_total, n.retrans_segs)),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Length(12),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(12),
        ratatui::layout::Constraint::Length(12),
    ];

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}
