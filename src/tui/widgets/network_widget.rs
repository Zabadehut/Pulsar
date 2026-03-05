use crate::collectors::NetworkMetrics;
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
    networks: &[NetworkMetrics],
    locale: Locale,
    detailed: bool,
    theme: &Theme,
    highlighted: bool,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            text(locale, " ◉ RESEAU ", " ◉ NETWORK "),
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
        frame.render_widget(
            Paragraph::new(text(locale, "Pas de donnees reseau", "No network data")),
            inner,
        );
        return;
    }

    let mut header_cells = vec![
        Cell::from(text(locale, "Interface", "Interface")),
        Cell::from(text(locale, "Topo", "Topo")),
        Cell::from(text(locale, "Fam", "Fam")),
        Cell::from(text(locale, "Media", "Media")),
        Cell::from("RX KB/s"),
        Cell::from("TX KB/s"),
        Cell::from("RX pkt/s"),
        Cell::from("TX pkt/s"),
        Cell::from(text(locale, "Erreurs", "Errors")),
        Cell::from(text(locale, "Pertes", "Drops")),
        Cell::from("TCP"),
        Cell::from("UDP/Rtx"),
    ];
    if detailed {
        header_cells.push(Cell::from(text(locale, "Conn", "Conn")));
    }
    let header = Row::new(header_cells).style(theme.highlight_style());

    let rows: Vec<Row> = networks
        .iter()
        .map(|n| {
            let mut cells = vec![
                Cell::from(n.interface.clone()),
                Cell::from(n.topology_hint.chars().take(10).collect::<String>()),
                Cell::from(n.family_hint.chars().take(9).collect::<String>()),
                Cell::from(n.medium_hint.chars().take(8).collect::<String>()),
                Cell::from(format!("{}", n.rx_bytes_sec / 1024)),
                Cell::from(format!("{}", n.tx_bytes_sec / 1024)),
                Cell::from(format!("{}", n.rx_packets_sec)),
                Cell::from(format!("{}", n.tx_packets_sec)),
                Cell::from(format!("{}", n.rx_errors + n.tx_errors)),
                Cell::from(format!("{}", n.rx_dropped + n.tx_dropped)),
                Cell::from(if n.tcp_state_breakdown_supported {
                    format!(
                        "{}/{}/{}",
                        n.connections_established, n.tcp_listen, n.tcp_time_wait
                    )
                } else {
                    format!("{}/n/a/n/a", n.connections_established)
                }),
                Cell::from(format!(
                    "{}/{}",
                    if n.udp_breakdown_supported {
                        n.udp_total.to_string()
                    } else {
                        "n/a".to_string()
                    },
                    if n.retrans_supported {
                        n.retrans_segs.to_string()
                    } else {
                        "n/a".to_string()
                    }
                )),
            ];
            if detailed {
                cells.push(Cell::from(format!("{}", n.connections_total)));
            }
            Row::new(cells)
        })
        .collect();

    let mut widths = vec![
        ratatui::layout::Constraint::Length(12),
        ratatui::layout::Constraint::Length(11),
        ratatui::layout::Constraint::Length(10),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(9),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(7),
        ratatui::layout::Constraint::Length(12),
        ratatui::layout::Constraint::Length(12),
    ];
    if detailed {
        widths.push(ratatui::layout::Constraint::Length(6));
    }

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);
}
