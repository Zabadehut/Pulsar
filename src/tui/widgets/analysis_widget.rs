use crate::collectors::{
    process::ProcessState, DiskMetrics, NetworkMetrics, ProcessMetrics, Snapshot,
};
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialistView {
    None,
    Pressure,
    Network,
    Jvm,
    DiskPressure,
}

impl SpecialistView {
    pub fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::None => text(locale, "aucun", "none"),
            Self::Pressure => text(locale, "pression+", "pressure+"),
            Self::Network => text(locale, "reseau+", "network+"),
            Self::Jvm => text(locale, "jvm+", "jvm+"),
            Self::DiskPressure => text(locale, "disque+", "disk+"),
        }
    }
}

pub fn summary_height(detailed: bool) -> u16 {
    if detailed {
        6
    } else {
        5
    }
}

pub fn render_summary(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    specialist: SpecialistView,
    locale: Locale,
    theme: &Theme,
) {
    if specialist == SpecialistView::None {
        return;
    }

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(
                " ◉ {} {} ",
                text(locale, "ANALYSE", "ANALYSIS"),
                specialist.label(locale)
            ),
            theme.title_style(),
        )]))
        .borders(Borders::ALL)
        .border_style(theme.border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let lines = match specialist {
        SpecialistView::Pressure => pressure_summary_lines(snapshot, locale, theme),
        SpecialistView::Network => network_summary_lines(snapshot, locale, theme),
        SpecialistView::Jvm => jvm_summary_lines(snapshot, locale, theme),
        SpecialistView::DiskPressure => disk_summary_lines(snapshot, locale, theme),
        SpecialistView::None => Vec::new(),
    };

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_drilldown(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    specialist: SpecialistView,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    if specialist == SpecialistView::None || area.height == 0 || area.width == 0 {
        return;
    }

    match specialist {
        SpecialistView::Pressure => {
            render_pressure_drilldown(frame, area, snapshot, locale, detailed, theme)
        }
        SpecialistView::Network => {
            render_network_drilldown(frame, area, snapshot, locale, detailed, theme)
        }
        SpecialistView::Jvm => render_jvm_drilldown(frame, area, snapshot, locale, detailed, theme),
        SpecialistView::DiskPressure => {
            render_disk_drilldown(frame, area, snapshot, locale, detailed, theme)
        }
        SpecialistView::None => {}
    }
}

fn render_pressure_drilldown(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(56), Constraint::Percentage(44)])
        .split(area);

    let mut rows = vec![
        key_value_row(
            text(locale, "Host mem pressure", "Host mem pressure"),
            format!("{:.0}%", snapshot.computed.memory_pressure * 100.0),
            severity_style(snapshot.computed.memory_pressure * 100.0, 90.0, 75.0, theme),
        ),
        key_value_row(
            text(locale, "Memory used", "Memory used"),
            snapshot
                .memory
                .as_ref()
                .map(|m| format!("{:.0}% ({:.1} GB)", m.usage_pct, kb_to_gb(m.used_kb)))
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Available", "Available"),
            snapshot
                .memory
                .as_ref()
                .map(|m| format!("{:.1} GB", kb_to_gb(m.available_kb)))
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Swap used", "Swap used"),
            snapshot
                .memory
                .as_ref()
                .map(|m| format!("{:.1} GB", kb_to_gb(m.swap_used_kb)))
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
    ];

    let psi = snapshot.linux.as_ref().and_then(|linux| linux.psi.as_ref());
    rows.extend([
        key_value_row(
            "PSI cpu avg10",
            format!(
                "{:.1}%",
                psi.and_then(|psi| psi.cpu.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0)
            ),
            body_style(theme),
        ),
        key_value_row(
            "PSI mem avg10",
            format!(
                "{:.1}%",
                psi.and_then(|psi| psi.memory.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0)
            ),
            severity_style(
                psi.and_then(|psi| psi.memory.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0),
                10.0,
                3.0,
                theme,
            ),
        ),
        key_value_row(
            "PSI io avg10",
            format!(
                "{:.1}%",
                psi.and_then(|psi| psi.io.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0)
            ),
            severity_style(
                psi.and_then(|psi| psi.io.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0),
                10.0,
                3.0,
                theme,
            ),
        ),
    ]);

    if let Some(cpu) = snapshot.cpu.as_ref() {
        let hottest_core = cpu.per_core.iter().max_by(|a, b| {
            a.usage_pct
                .partial_cmp(&b.usage_pct)
                .unwrap_or(Ordering::Equal)
        });
        rows.push(key_value_row(
            text(locale, "Hot core", "Hot core"),
            hottest_core
                .map(|core| format!("#{} {:.1}%", core.id, core.usage_pct))
                .unwrap_or_else(|| "-".to_string()),
            hottest_core
                .map(|core| severity_style(core.usage_pct, 90.0, 75.0, theme))
                .unwrap_or_else(|| body_style(theme)),
        ));
        rows.push(key_value_row(
            text(locale, "Load 1/5/15", "Load 1/5/15"),
            format!(
                "{:.2} / {:.2} / {:.2}",
                cpu.load_avg_1, cpu.load_avg_5, cpu.load_avg_15
            ),
            body_style(theme),
        ));
    }

    if let Some(cgroup) = snapshot
        .linux
        .as_ref()
        .and_then(|linux| linux.cgroup.as_ref())
    {
        rows.push(key_value_row(
            text(locale, "Cgroup mem", "Cgroup mem"),
            format!("{:.1}%", cgroup.memory_usage_pct),
            severity_style(cgroup.memory_usage_pct, 90.0, 75.0, theme),
        ));
        let throttled_pct = if cgroup.cpu_nr_periods == 0 {
            0.0
        } else {
            cgroup.cpu_nr_throttled as f64 / cgroup.cpu_nr_periods as f64 * 100.0
        };
        rows.push(key_value_row(
            text(locale, "CPU throttled", "CPU throttled"),
            format!("{:.1}%", throttled_pct),
            severity_style(throttled_pct, 20.0, 5.0, theme),
        ));
    }

    let signal_table = metric_table(
        text(locale, " ◉ CHEMINS DE PRESSION ", " ◉ PRESSURE PATHS "),
        vec![
            Cell::from(text(locale, "Signal", "Signal")),
            Cell::from(text(locale, "Valeur", "Value")),
        ],
        rows,
        [Constraint::Percentage(56), Constraint::Percentage(44)],
        theme,
    );
    frame.render_widget(signal_table, cols[0]);

    let right_rows = if detailed {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Percentage(30),
                Constraint::Percentage(28),
            ])
            .split(cols[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(cols[1])
    };

    render_process_pressure_table(frame, right_rows[0], snapshot, locale, detailed, theme);
    if detailed {
        render_pressure_lens_table(frame, right_rows[1], snapshot, locale, theme);
        render_focus_notes(
            frame,
            right_rows[2],
            text(locale, " ◉ LECTURE ", " ◉ READING "),
            pressure_focus_lines(snapshot, locale, theme),
            theme,
        );
    } else {
        render_focus_notes(
            frame,
            right_rows[1],
            text(locale, " ◉ LECTURE ", " ◉ READING "),
            pressure_focus_lines(snapshot, locale, theme),
            theme,
        );
    }
}

fn render_network_drilldown(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_interface_table(frame, cols[0], snapshot, locale, detailed, theme);

    let right_rows = if detailed {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(44),
                Constraint::Percentage(28),
                Constraint::Percentage(28),
            ])
            .split(cols[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(cols[1])
    };

    render_socket_state_table(frame, right_rows[0], snapshot, locale, theme);
    if detailed {
        render_network_lens_table(frame, right_rows[1], snapshot, locale, theme);
        render_focus_notes(
            frame,
            right_rows[2],
            text(locale, " ◉ ANALYSE LIEN ", " ◉ LINK ANALYSIS "),
            network_focus_lines(snapshot, locale, theme),
            theme,
        );
    } else {
        render_focus_notes(
            frame,
            right_rows[1],
            text(locale, " ◉ ANALYSE LIEN ", " ◉ LINK ANALYSIS "),
            network_focus_lines(snapshot, locale, theme),
            theme,
        );
    }
}

fn render_jvm_drilldown(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    render_jvm_table(frame, cols[0], snapshot, locale, detailed, theme);

    let right_rows = if detailed {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(36),
                Constraint::Percentage(34),
                Constraint::Percentage(30),
            ])
            .split(cols[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(cols[1])
    };

    render_jvm_hotspots(frame, right_rows[0], snapshot, locale, theme);
    if detailed {
        render_jvm_profile_table(frame, right_rows[1], snapshot, locale, theme);
        render_focus_notes(
            frame,
            right_rows[2],
            text(locale, " ◉ THREAD / RUNTIME ", " ◉ THREAD / RUNTIME "),
            jvm_focus_lines(snapshot, locale, theme),
            theme,
        );
    } else {
        render_focus_notes(
            frame,
            right_rows[1],
            text(locale, " ◉ THREAD / RUNTIME ", " ◉ THREAD / RUNTIME "),
            jvm_focus_lines(snapshot, locale, theme),
            theme,
        );
    }
}

fn render_disk_drilldown(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_disk_table(frame, cols[0], snapshot, locale, detailed, theme);

    let right_rows = if detailed {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Percentage(30),
                Constraint::Percentage(28),
            ])
            .split(cols[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(cols[1])
    };

    render_disk_waiter_table(frame, right_rows[0], snapshot, locale, detailed, theme);
    if detailed {
        render_disk_lens_table(frame, right_rows[1], snapshot, locale, theme);
        render_focus_notes(
            frame,
            right_rows[2],
            text(locale, " ◉ CONTENTION ", " ◉ CONTENTION "),
            disk_focus_lines(snapshot, locale, theme),
            theme,
        );
    } else {
        render_focus_notes(
            frame,
            right_rows[1],
            text(locale, " ◉ CONTENTION ", " ◉ CONTENTION "),
            disk_focus_lines(snapshot, locale, theme),
            theme,
        );
    }
}

fn render_process_pressure_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let mut processes: Vec<&ProcessMetrics> = snapshot.processes.iter().collect();
    processes.sort_by(|a, b| {
        process_pressure_score(b)
            .cmp(&process_pressure_score(a))
            .then_with(|| b.threads.cmp(&a.threads))
    });

    let rows = processes
        .into_iter()
        .take(if detailed { 8 } else { 5 })
        .map(|proc| {
            Row::new(vec![
                Cell::from(proc.pid.to_string()),
                Cell::from(truncate(&proc.name, 14)),
                Cell::from(match proc.state {
                    ProcessState::Running => "R",
                    ProcessState::Sleeping => "S",
                    ProcessState::DiskSleep => "D",
                    ProcessState::Zombie => "Z",
                    ProcessState::Stopped => "T",
                    ProcessState::TracingStop => "t",
                    ProcessState::Unknown => "?",
                }),
                Cell::from(format!("{:.1}", proc.cpu_pct)),
                Cell::from(format!("{:.0}", proc.mem_rss_kb as f64 / 1024.0)),
                Cell::from(proc.threads.to_string()),
            ])
        })
        .collect::<Vec<_>>();

    let table = metric_table(
        text(
            locale,
            " ◉ PROCESSUS SOUS PRESSION ",
            " ◉ PRESSURED PROCESSES ",
        ),
        vec![
            Cell::from("PID"),
            Cell::from(text(locale, "Nom", "Name")),
            Cell::from(text(locale, "Etat", "State")),
            Cell::from("CPU%"),
            Cell::from("RSS"),
            Cell::from("Thr"),
        ],
        rows,
        [
            Constraint::Length(7),
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(5),
        ],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_pressure_lens_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) {
    let memory = snapshot.memory.as_ref();
    let cgroup = snapshot
        .linux
        .as_ref()
        .and_then(|linux| linux.cgroup.as_ref());
    let psi = snapshot.linux.as_ref().and_then(|linux| linux.psi.as_ref());
    let cpu = snapshot.cpu.as_ref();
    let disk_sleep = snapshot
        .processes
        .iter()
        .filter(|proc| proc.state == ProcessState::DiskSleep)
        .count() as f64;

    let reclaim_score = memory
        .map(|mem| mem.vm_pgscan.saturating_add(mem.vm_pgsteal) as f64 / 1_000.0)
        .unwrap_or(0.0);
    let swap_push = memory
        .map(|mem| mem.vm_pswpin.saturating_add(mem.vm_pswpout) as f64)
        .unwrap_or(0.0);
    let host_vs_cgroup_gap = cgroup
        .map(|cg| (snapshot.computed.memory_pressure * 100.0 - cg.memory_usage_pct).abs())
        .unwrap_or(0.0);
    let cpu_wait = cpu.map(|entry| entry.iowait_pct).unwrap_or(0.0)
        + psi
            .and_then(|entry| entry.cpu.some.as_ref().map(|window| window.avg10))
            .unwrap_or(0.0);
    let io_stall = psi
        .and_then(|entry| entry.io.some.as_ref().map(|window| window.avg10))
        .unwrap_or(0.0)
        + disk_sleep * 2.0;
    let mem_stall = psi
        .and_then(|entry| entry.memory.some.as_ref().map(|window| window.avg10))
        .unwrap_or(0.0)
        + snapshot.computed.memory_pressure * 100.0 / 10.0;

    let rows = vec![
        key_value_row(
            text(locale, "Reclaim score", "Reclaim score"),
            format!("{reclaim_score:.1}"),
            severity_style(reclaim_score, 50.0, 5.0, theme),
        ),
        key_value_row(
            text(locale, "Swap push", "Swap push"),
            format!("{swap_push:.0}"),
            severity_style(swap_push, 10.0, 1.0, theme),
        ),
        key_value_row(
            text(locale, "Host/cgroup gap", "Host/cgroup gap"),
            format!("{host_vs_cgroup_gap:.0}%"),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "CPU wait mix", "CPU wait mix"),
            format!("{cpu_wait:.1}"),
            severity_style(cpu_wait, 15.0, 3.0, theme),
        ),
        key_value_row(
            text(locale, "IO stall mix", "IO stall mix"),
            format!("{io_stall:.1}"),
            severity_style(io_stall, 15.0, 3.0, theme),
        ),
        key_value_row(
            text(locale, "Mem stall mix", "Mem stall mix"),
            format!("{mem_stall:.1}"),
            severity_style(mem_stall, 15.0, 5.0, theme),
        ),
    ];

    let table = metric_table(
        text(locale, " ◉ PRESSURE LENSES ", " ◉ PRESSURE LENSES "),
        vec![
            Cell::from(text(locale, "Signal", "Signal")),
            Cell::from(text(locale, "Valeur", "Value")),
        ],
        rows,
        [Constraint::Percentage(58), Constraint::Percentage(42)],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_interface_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let mut ifaces: Vec<&NetworkMetrics> = snapshot.networks.iter().collect();
    ifaces.sort_by_key(|net| {
        std::cmp::Reverse(
            net.rx_bytes_sec + net.tx_bytes_sec + ((net.rx_errors + net.tx_errors) * 1024),
        )
    });

    let rows = ifaces
        .into_iter()
        .take(if detailed { 8 } else { 5 })
        .map(|net| {
            let total_pps = net.rx_packets_sec + net.tx_packets_sec;
            let mut cells = vec![
                Cell::from(truncate(&net.interface, 10)),
                Cell::from((net.rx_bytes_sec / 1024).to_string()),
                Cell::from((net.tx_bytes_sec / 1024).to_string()),
                Cell::from(total_pps.to_string()),
                Cell::from((net.rx_errors + net.tx_errors).to_string()),
                Cell::from((net.rx_dropped + net.tx_dropped).to_string()),
            ];
            if detailed {
                cells.push(Cell::from(net.retrans_segs.to_string()));
            }
            Row::new(cells)
        })
        .collect::<Vec<_>>();

    let mut widths = vec![
        Constraint::Length(11),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(7),
        Constraint::Length(7),
    ];
    let mut header = vec![
        Cell::from(text(locale, "Iface", "Iface")),
        Cell::from("RX KB"),
        Cell::from("TX KB"),
        Cell::from("PPS"),
        Cell::from("Err"),
        Cell::from("Drop"),
    ];
    if detailed {
        widths.push(Constraint::Length(7));
        header.push(Cell::from("Rtx"));
    }

    let table = metric_table(
        text(locale, " ◉ INTERFACES ", " ◉ INTERFACES "),
        header,
        rows,
        widths,
        theme,
    );
    frame.render_widget(table, area);
}

fn render_socket_state_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) {
    let state = snapshot.networks.first();
    let rows = vec![
        key_value_row(
            text(locale, "Established", "Established"),
            state
                .map(|net| net.connections_established.to_string())
                .unwrap_or_else(|| "0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "Listen",
            state
                .map(|net| net.tcp_listen.to_string())
                .unwrap_or_else(|| "0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "TimeWait",
            state
                .map(|net| net.tcp_time_wait.to_string())
                .unwrap_or_else(|| "0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "Syn sent/recv",
            state
                .map(|net| format!("{}/{}", net.tcp_syn_sent, net.tcp_syn_recv))
                .unwrap_or_else(|| "0/0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "Fin wait1/2",
            state
                .map(|net| format!("{}/{}", net.tcp_fin_wait1, net.tcp_fin_wait2))
                .unwrap_or_else(|| "0/0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "Close wait",
            state
                .map(|net| net.tcp_close_wait.to_string())
                .unwrap_or_else(|| "0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "UDP total",
            state
                .map(|net| net.udp_total.to_string())
                .unwrap_or_else(|| "0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "UDP est",
            state
                .map(|net| net.udp_established.to_string())
                .unwrap_or_else(|| "0".to_string()),
            body_style(theme),
        ),
        key_value_row(
            "Retrans",
            state
                .map(|net| net.retrans_segs.to_string())
                .unwrap_or_else(|| "0".to_string()),
            state
                .map(|net| severity_style(net.retrans_segs as f64, 100.0, 10.0, theme))
                .unwrap_or_else(|| body_style(theme)),
        ),
    ];

    let table = metric_table(
        text(locale, " ◉ ETATS SOCKET/TCP ", " ◉ SOCKET/TCP STATES "),
        vec![
            Cell::from(text(locale, "Etat", "State")),
            Cell::from(text(locale, "Valeur", "Value")),
        ],
        rows,
        [Constraint::Percentage(55), Constraint::Percentage(45)],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_network_lens_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) {
    let state = snapshot.networks.first();
    let (handshake, closing, listeners, active_ratio, udp_mix, loss_path) = if let Some(net) = state
    {
        let closing = net.tcp_fin_wait1
            + net.tcp_fin_wait2
            + net.tcp_close
            + net.tcp_close_wait
            + net.tcp_last_ack
            + net.tcp_closing
            + net.tcp_time_wait;
        let total_tcpish = (net.connections_total.max(1)) as f64;
        (
            net.tcp_syn_sent + net.tcp_syn_recv,
            closing,
            net.tcp_listen + net.udp_total,
            net.connections_established as f64 / total_tcpish * 100.0,
            if net.connections_total == 0 {
                0.0
            } else {
                net.udp_total as f64 / net.connections_total as f64 * 100.0
            },
            net.retrans_segs
                + snapshot
                    .networks
                    .iter()
                    .map(|entry| {
                        entry.rx_errors + entry.tx_errors + entry.rx_dropped + entry.tx_dropped
                    })
                    .sum::<u64>(),
        )
    } else {
        (0, 0, 0, 0.0, 0.0, 0)
    };

    let rows = vec![
        key_value_row(
            text(locale, "Handshake pressure", "Handshake pressure"),
            handshake.to_string(),
            severity_style(handshake as f64, 20.0, 5.0, theme),
        ),
        key_value_row(
            text(locale, "Closing backlog", "Closing backlog"),
            closing.to_string(),
            severity_style(closing as f64, 50.0, 10.0, theme),
        ),
        key_value_row(
            text(locale, "Listener surface", "Listener surface"),
            listeners.to_string(),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Active ratio", "Active ratio"),
            format!("{active_ratio:.0}%"),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "UDP mix", "UDP mix"),
            format!("{udp_mix:.0}%"),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Loss path score", "Loss path score"),
            loss_path.to_string(),
            severity_style(loss_path as f64, 200.0, 20.0, theme),
        ),
    ];

    let table = metric_table(
        text(locale, " ◉ SESSION LENSES ", " ◉ SESSION LENSES "),
        vec![
            Cell::from(text(locale, "Signal", "Signal")),
            Cell::from(text(locale, "Valeur", "Value")),
        ],
        rows,
        [Constraint::Percentage(58), Constraint::Percentage(42)],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_jvm_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let mut jvms: Vec<&ProcessMetrics> = snapshot
        .processes
        .iter()
        .filter(|proc| proc.is_jvm)
        .collect();
    jvms.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.threads.cmp(&a.threads))
    });

    let rows = jvms
        .into_iter()
        .take(if detailed { 8 } else { 5 })
        .map(|proc| {
            let mut cells = vec![
                Cell::from(proc.pid.to_string()),
                Cell::from(truncate(&proc.name, 16)),
                Cell::from(format!("{:.1}", proc.cpu_pct)),
                Cell::from(format!("{:.0}", proc.mem_rss_kb as f64 / 1024.0)),
                Cell::from(proc.threads.to_string()),
                Cell::from(proc.fd_count.to_string()),
            ];
            if detailed {
                cells.push(Cell::from(format!(
                    "{:.1}",
                    (proc.io_read_bytes + proc.io_write_bytes) as f64 / (1024.0 * 1024.0)
                )));
            }
            Row::new(cells)
        })
        .collect::<Vec<_>>();

    let mut widths = vec![
        Constraint::Length(7),
        Constraint::Percentage(34),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(6),
    ];
    let mut header = vec![
        Cell::from("PID"),
        Cell::from(text(locale, "Nom", "Name")),
        Cell::from("CPU%"),
        Cell::from("RSS"),
        Cell::from("Thr"),
        Cell::from("FD"),
    ];
    if detailed {
        widths.push(Constraint::Length(7));
        header.push(Cell::from("IO"));
    }

    let table = metric_table(
        text(locale, " ◉ JVM HOTSPOTS ", " ◉ JVM HOTSPOTS "),
        header,
        rows,
        widths,
        theme,
    );
    frame.render_widget(table, area);
}

fn render_jvm_hotspots(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) {
    let jvms: Vec<&ProcessMetrics> = snapshot
        .processes
        .iter()
        .filter(|proc| proc.is_jvm)
        .collect();
    let top_cpu = jvms
        .iter()
        .copied()
        .max_by(|a, b| a.cpu_pct.partial_cmp(&b.cpu_pct).unwrap_or(Ordering::Equal));
    let top_mem = jvms.iter().copied().max_by_key(|proc| proc.mem_rss_kb);
    let top_threads = jvms.iter().copied().max_by_key(|proc| proc.threads);
    let top_fds = jvms.iter().copied().max_by_key(|proc| proc.fd_count);
    let total_io_mb = jvms
        .iter()
        .map(|proc| proc.io_read_bytes + proc.io_write_bytes)
        .sum::<u64>() as f64
        / (1024.0 * 1024.0);

    let rows = vec![
        key_value_row(
            text(locale, "JVM count", "JVM count"),
            jvms.len().to_string(),
            if jvms.is_empty() {
                theme.muted_style()
            } else {
                theme.highlight_style()
            },
        ),
        key_value_row(
            text(locale, "Top CPU", "Top CPU"),
            top_cpu
                .map(|proc| format!("{} {:.1}%", truncate(&proc.name, 14), proc.cpu_pct))
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Top RSS", "Top RSS"),
            top_mem
                .map(|proc| {
                    format!(
                        "{} {:.0} MB",
                        truncate(&proc.name, 14),
                        proc.mem_rss_kb as f64 / 1024.0
                    )
                })
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Most threads", "Most threads"),
            top_threads
                .map(|proc| format!("{} {}", truncate(&proc.name, 14), proc.threads))
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Top FDs", "Top FDs"),
            top_fds
                .map(|proc| format!("{} {}", truncate(&proc.name, 14), proc.fd_count))
                .unwrap_or_else(|| "-".to_string()),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Total IO", "Total IO"),
            format!("{total_io_mb:.1} MB"),
            body_style(theme),
        ),
    ];

    let table = metric_table(
        text(locale, " ◉ RUNTIME FOCUS ", " ◉ RUNTIME FOCUS "),
        vec![
            Cell::from(text(locale, "Signal", "Signal")),
            Cell::from(text(locale, "Valeur", "Value")),
        ],
        rows,
        [Constraint::Percentage(52), Constraint::Percentage(48)],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_jvm_profile_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) {
    let mut jvms: Vec<&ProcessMetrics> = snapshot
        .processes
        .iter()
        .filter(|proc| proc.is_jvm)
        .collect();
    jvms.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.mem_rss_kb.cmp(&a.mem_rss_kb))
    });

    let rows = jvms
        .into_iter()
        .take(4)
        .map(|proc| {
            Row::new(vec![
                Cell::from(truncate(&proc.name, 10)),
                Cell::from(jvm_role(proc, locale)),
                Cell::from(jvm_dominant_pressure(proc, locale)),
                Cell::from(jvm_heap_hint(proc)),
            ])
        })
        .collect::<Vec<_>>();

    let table = metric_table(
        text(locale, " ◉ PROFILS JVM ", " ◉ JVM PROFILES "),
        vec![
            Cell::from(text(locale, "Nom", "Name")),
            Cell::from(text(locale, "Role", "Role")),
            Cell::from(text(locale, "Pression", "Pressure")),
            Cell::from(text(locale, "Heap", "Heap")),
        ],
        rows,
        [
            Constraint::Percentage(26),
            Constraint::Percentage(26),
            Constraint::Percentage(28),
            Constraint::Percentage(20),
        ],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_disk_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let mut disks: Vec<&DiskMetrics> = snapshot.disks.iter().collect();
    disks.sort_by(|a, b| {
        b.util_pct
            .partial_cmp(&a.util_pct)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                b.await_ms
                    .partial_cmp(&a.await_ms)
                    .unwrap_or(Ordering::Equal)
            })
    });

    let rows = disks
        .into_iter()
        .take(if detailed { 8 } else { 5 })
        .map(|disk| {
            let mut cells = vec![
                Cell::from(truncate(&disk.device, 8)),
                Cell::from(truncate(&disk.structure_hint, 10)),
                Cell::from(truncate(&disk.protocol_hint, 10)),
                Cell::from(truncate(&disk.media_hint, 8)),
                Cell::from(truncate(&disk.mount_point, 10)),
                Cell::from(format!("{:.1}", disk.util_pct)),
                Cell::from(format!("{:.1}", disk.await_ms)),
                Cell::from(format!("{:.2}", disk.queue_depth)),
                Cell::from(format!("{}", disk.read_iops + disk.write_iops)),
            ];
            if detailed {
                cells.push(Cell::from(format!(
                    "{}",
                    disk.read_throughput_kb + disk.write_throughput_kb
                )));
            }
            Row::new(cells)
        })
        .collect::<Vec<_>>();

    let mut widths = vec![
        Constraint::Length(9),
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(9),
        Constraint::Length(11),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(7),
        Constraint::Length(8),
    ];
    let mut header = vec![
        Cell::from(text(locale, "Disk", "Disk")),
        Cell::from(text(locale, "Struct", "Struct")),
        Cell::from(text(locale, "Proto", "Proto")),
        Cell::from(text(locale, "Media", "Media")),
        Cell::from(text(locale, "Mount", "Mount")),
        Cell::from("Util"),
        Cell::from("Await"),
        Cell::from("Qd"),
        Cell::from("IOPS"),
    ];
    if detailed {
        widths.push(Constraint::Length(9));
        header.push(Cell::from("KB/s"));
    }

    let table = metric_table(
        text(locale, " ◉ DISQUES CHAUDS ", " ◉ HOT DISKS "),
        header,
        rows,
        widths,
        theme,
    );
    frame.render_widget(table, area);
}

fn render_disk_waiter_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    detailed: bool,
    theme: &Theme,
) {
    let mut processes: Vec<&ProcessMetrics> = snapshot.processes.iter().collect();
    processes.sort_by(|a, b| {
        (b.io_read_bytes + b.io_write_bytes)
            .cmp(&(a.io_read_bytes + a.io_write_bytes))
            .then_with(|| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(Ordering::Equal))
    });

    let rows = processes
        .into_iter()
        .take(if detailed { 8 } else { 5 })
        .map(|proc| {
            Row::new(vec![
                Cell::from(proc.pid.to_string()),
                Cell::from(truncate(&proc.name, 12)),
                Cell::from(match proc.state {
                    ProcessState::DiskSleep => "D",
                    ProcessState::Running => "R",
                    ProcessState::Sleeping => "S",
                    ProcessState::Zombie => "Z",
                    ProcessState::Stopped => "T",
                    ProcessState::TracingStop => "t",
                    ProcessState::Unknown => "?",
                }),
                Cell::from(format!(
                    "{:.1}",
                    proc.io_read_bytes as f64 / (1024.0 * 1024.0)
                )),
                Cell::from(format!(
                    "{:.1}",
                    proc.io_write_bytes as f64 / (1024.0 * 1024.0)
                )),
                Cell::from(format!("{:.1}", proc.cpu_pct)),
            ])
        })
        .collect::<Vec<_>>();

    let table = metric_table(
        text(locale, " ◉ ATTENTEURS / IO ", " ◉ WAITERS / IO "),
        vec![
            Cell::from("PID"),
            Cell::from(text(locale, "Nom", "Name")),
            Cell::from(text(locale, "Etat", "State")),
            Cell::from("R MB"),
            Cell::from("W MB"),
            Cell::from("CPU%"),
        ],
        rows,
        [
            Constraint::Length(7),
            Constraint::Percentage(36),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(7),
        ],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_disk_lens_table(
    frame: &mut Frame,
    area: Rect,
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) {
    let hottest = snapshot.disks.iter().max_by(|a, b| {
        b.util_pct
            .partial_cmp(&a.util_pct)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                b.await_ms
                    .partial_cmp(&a.await_ms)
                    .unwrap_or(Ordering::Equal)
            })
    });
    let total_iops = snapshot
        .disks
        .iter()
        .map(|disk| disk.read_iops + disk.write_iops)
        .sum::<u64>() as f64;
    let total_throughput_kb = snapshot
        .disks
        .iter()
        .map(|disk| disk.read_throughput_kb + disk.write_throughput_kb)
        .sum::<u64>() as f64;
    let disk_sleep = snapshot
        .processes
        .iter()
        .filter(|proc| proc.state == ProcessState::DiskSleep)
        .count() as f64;
    let top_io = snapshot
        .processes
        .iter()
        .map(|proc| proc.io_read_bytes + proc.io_write_bytes)
        .sum::<u64>() as f64
        / (1024.0 * 1024.0);

    let rows = vec![
        key_value_row(
            text(locale, "Busy path", "Busy path"),
            hottest
                .map(|disk| format!("{:.1}%", disk.util_pct))
                .unwrap_or_else(|| "0.0%".to_string()),
            hottest
                .map(|disk| severity_style(disk.util_pct, 90.0, 75.0, theme))
                .unwrap_or_else(|| body_style(theme)),
        ),
        key_value_row(
            text(locale, "Latency path", "Latency path"),
            hottest
                .map(|disk| format!("{:.1} ms", disk.await_ms))
                .unwrap_or_else(|| "0.0 ms".to_string()),
            hottest
                .map(|disk| severity_style(disk.await_ms, 20.0, 5.0, theme))
                .unwrap_or_else(|| body_style(theme)),
        ),
        key_value_row(
            text(locale, "Queue path", "Queue path"),
            hottest
                .map(|disk| format!("{:.2}", disk.queue_depth))
                .unwrap_or_else(|| "0.00".to_string()),
            hottest
                .map(|disk| severity_style(disk.queue_depth, 2.0, 0.5, theme))
                .unwrap_or_else(|| body_style(theme)),
        ),
        key_value_row(
            text(locale, "IOPS sum", "IOPS sum"),
            format!("{total_iops:.0}"),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Throughput sum", "Throughput sum"),
            format!("{:.0} KB/s", total_throughput_kb),
            body_style(theme),
        ),
        key_value_row(
            text(locale, "Waiter pressure", "Waiter pressure"),
            format!("D {:.0} / IO {:.1} MB", disk_sleep, top_io),
            severity_style(disk_sleep + top_io / 100.0, 5.0, 1.0, theme),
        ),
    ];

    let table = metric_table(
        text(locale, " ◉ DISK LENSES ", " ◉ DISK LENSES "),
        vec![
            Cell::from(text(locale, "Signal", "Signal")),
            Cell::from(text(locale, "Valeur", "Value")),
        ],
        rows,
        [Constraint::Percentage(58), Constraint::Percentage(42)],
        theme,
    );
    frame.render_widget(table, area);
}

fn render_focus_notes(
    frame: &mut Frame,
    area: Rect,
    title: &'static str,
    lines: Vec<Line<'static>>,
    theme: &Theme,
) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(title, theme.title_style())]))
        .borders(Borders::ALL)
        .border_style(theme.border_style());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn pressure_summary_lines(
    snapshot: &Snapshot,
    locale: Locale,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let mem_pressure = snapshot.computed.memory_pressure * 100.0;
    let (psi_cpu, psi_mem, psi_io, cgroup_mem, throttle_pct) =
        if let Some(linux) = snapshot.linux.as_ref() {
            (
                linux
                    .psi
                    .as_ref()
                    .and_then(|psi| psi.cpu.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0),
                linux
                    .psi
                    .as_ref()
                    .and_then(|psi| psi.memory.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0),
                linux
                    .psi
                    .as_ref()
                    .and_then(|psi| psi.io.some.as_ref().map(|v| v.avg10))
                    .unwrap_or(0.0),
                linux
                    .cgroup
                    .as_ref()
                    .map(|c| c.memory_usage_pct)
                    .unwrap_or(0.0),
                linux
                    .cgroup
                    .as_ref()
                    .map(|c| {
                        if c.cpu_nr_periods == 0 {
                            0.0
                        } else {
                            c.cpu_nr_throttled as f64 / c.cpu_nr_periods as f64 * 100.0
                        }
                    })
                    .unwrap_or(0.0),
            )
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        };
    let disk_sleep = snapshot
        .processes
        .iter()
        .filter(|proc| proc.state == ProcessState::DiskSleep)
        .count();
    let hot_pressure_process = snapshot.processes.iter().max_by_key(|proc| proc.threads);

    vec![
        Line::from(vec![
            Span::styled(
                format!(
                    "{} {:.0}%  ",
                    text(locale, "memoire", "memory"),
                    mem_pressure
                ),
                style_for_pressure(mem_pressure, theme),
            ),
            Span::raw(format!(
                "psi cpu {:.1}%  psi mem {:.1}%  psi io {:.1}%",
                psi_cpu, psi_mem, psi_io
            )),
        ]),
        Line::from(vec![
            Span::raw(format!(
                "{} {:.1}%  ",
                text(locale, "cgroup mem", "cgroup mem"),
                cgroup_mem
            )),
            Span::raw(format!("throttle {:.1}%  ", throttle_pct)),
            Span::styled(
                format!(
                    "{} {}",
                    text(locale, "processus D:", "disk-sleep:"),
                    disk_sleep
                ),
                if disk_sleep > 0 {
                    theme.alert_style()
                } else {
                    theme.muted_style()
                },
            ),
        ]),
        Line::from(format!(
            "{} {}  {} {}",
            text(locale, "hot proc", "hot proc"),
            hot_pressure_process
                .map(|proc| proc.name.as_str())
                .unwrap_or("-"),
            text(locale, "threads", "threads"),
            hot_pressure_process.map(|proc| proc.threads).unwrap_or(0),
        )),
    ]
}

fn network_summary_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let total_rx_kb = snapshot
        .networks
        .iter()
        .map(|net| net.rx_bytes_sec / 1024)
        .sum::<u64>();
    let total_tx_kb = snapshot
        .networks
        .iter()
        .map(|net| net.tx_bytes_sec / 1024)
        .sum::<u64>();
    let errors = snapshot
        .networks
        .iter()
        .map(|net| net.rx_errors + net.tx_errors)
        .sum::<u64>();
    let drops = snapshot
        .networks
        .iter()
        .map(|net| net.rx_dropped + net.tx_dropped)
        .sum::<u64>();
    let hottest = snapshot
        .networks
        .iter()
        .max_by_key(|net| net.rx_bytes_sec + net.tx_bytes_sec);
    let state = snapshot.networks.first();
    let total_pps = snapshot
        .networks
        .iter()
        .map(|net| net.rx_packets_sec + net.tx_packets_sec)
        .sum::<u64>();
    let top_retrans = snapshot.networks.iter().max_by_key(|net| net.retrans_segs);

    vec![
        Line::from(vec![
            Span::raw(format!(
                "rx {} KB/s  tx {} KB/s  ",
                total_rx_kb, total_tx_kb
            )),
            Span::styled(
                format!(
                    "{} {} / {} {}",
                    text(locale, "erreurs", "errors"),
                    errors,
                    text(locale, "pertes", "drops"),
                    drops
                ),
                if errors + drops > 0 {
                    theme.alert_style()
                } else {
                    theme.muted_style()
                },
            ),
        ]),
        Line::from(format!(
            "{} {}  est {}  listen {}  tw {}  retrans {}",
            text(locale, "hot iface", "hot iface"),
            hottest.map(|net| net.interface.as_str()).unwrap_or("-"),
            state.map(|net| net.connections_established).unwrap_or(0),
            state.map(|net| net.tcp_listen).unwrap_or(0),
            state.map(|net| net.tcp_time_wait).unwrap_or(0),
            state.map(|net| net.retrans_segs).unwrap_or(0),
        )),
        Line::from(format!(
            "pps {}  {} {}  syn {}/{}",
            total_pps,
            text(locale, "top retrans", "top retrans"),
            top_retrans.map(|net| net.interface.as_str()).unwrap_or("-"),
            state.map(|net| net.tcp_syn_sent).unwrap_or(0),
            state.map(|net| net.tcp_syn_recv).unwrap_or(0),
        )),
    ]
}

fn jvm_summary_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let jvms: Vec<_> = snapshot
        .processes
        .iter()
        .filter(|proc| proc.is_jvm)
        .collect();
    let total_threads = jvms.iter().map(|proc| proc.threads).sum::<u32>();
    let total_fds = jvms.iter().map(|proc| proc.fd_count).sum::<u32>();
    let total_io_mb = jvms
        .iter()
        .map(|proc| proc.io_read_bytes + proc.io_write_bytes)
        .sum::<u64>() as f64
        / (1024.0 * 1024.0);
    let top_cpu = jvms
        .iter()
        .max_by(|a, b| a.cpu_pct.partial_cmp(&b.cpu_pct).unwrap_or(Ordering::Equal));
    let top_mem = jvms.iter().max_by_key(|proc| proc.mem_rss_kb);
    let most_threads = jvms.iter().max_by_key(|proc| proc.threads);

    vec![
        Line::from(vec![
            Span::styled(
                format!("{} {}  ", text(locale, "jvm", "jvm"), jvms.len()),
                if jvms.is_empty() {
                    theme.muted_style()
                } else {
                    theme.highlight_style()
                },
            ),
            Span::raw(format!(
                "{} {}  fds {}",
                text(locale, "threads", "threads"),
                total_threads,
                total_fds
            )),
        ]),
        Line::from(format!(
            "{} {} {:.1}%  {} {} {:.0} MB",
            text(locale, "top cpu", "top cpu"),
            top_cpu.map(|proc| proc.name.as_str()).unwrap_or("-"),
            top_cpu.map(|proc| proc.cpu_pct).unwrap_or(0.0),
            text(locale, "top mem", "top mem"),
            top_mem.map(|proc| proc.name.as_str()).unwrap_or("-"),
            top_mem
                .map(|proc| proc.mem_rss_kb as f64 / 1024.0)
                .unwrap_or(0.0),
        )),
        Line::from(format!(
            "{} {}  io {:.1} MB  {} {}",
            text(locale, "top threads", "top threads"),
            most_threads.map(|proc| proc.name.as_str()).unwrap_or("-"),
            total_io_mb,
            text(locale, "count", "count"),
            most_threads.map(|proc| proc.threads).unwrap_or(0),
        )),
    ]
}

fn disk_summary_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let hottest_disk = snapshot.disks.iter().max_by(|a, b| {
        a.util_pct
            .partial_cmp(&b.util_pct)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                a.await_ms
                    .partial_cmp(&b.await_ms)
                    .unwrap_or(Ordering::Equal)
            })
    });
    let disk_sleep = snapshot
        .processes
        .iter()
        .filter(|proc| proc.state == ProcessState::DiskSleep)
        .count();
    let top_writer = snapshot
        .processes
        .iter()
        .max_by_key(|proc| proc.io_write_bytes);
    let top_reader = snapshot
        .processes
        .iter()
        .max_by_key(|proc| proc.io_read_bytes);

    vec![
        Line::from(vec![
            Span::styled(
                format!(
                    "{} {}  ",
                    text(locale, "disque chaud", "hot disk"),
                    hottest_disk.map(|disk| disk.device.as_str()).unwrap_or("-")
                ),
                if hottest_disk.is_some_and(|disk| disk.util_pct >= 80.0 || disk.await_ms >= 20.0) {
                    theme.alert_style()
                } else {
                    theme.highlight_style()
                },
            ),
            Span::raw(format!(
                "util {:.1}%  await {:.1}ms  qd {:.2}",
                hottest_disk.map(|disk| disk.util_pct).unwrap_or(0.0),
                hottest_disk.map(|disk| disk.await_ms).unwrap_or(0.0),
                hottest_disk.map(|disk| disk.queue_depth).unwrap_or(0.0)
            )),
        ]),
        Line::from(format!(
            "{} {}  {} {}",
            text(locale, "processus D:", "disk-sleep:"),
            disk_sleep,
            text(locale, "top writer", "top writer"),
            top_writer.map(|proc| proc.name.as_str()).unwrap_or("-"),
        )),
        Line::from(format!(
            "{} {}  svc {:.1}ms",
            text(locale, "top reader", "top reader"),
            top_reader.map(|proc| proc.name.as_str()).unwrap_or("-"),
            hottest_disk.map(|disk| disk.service_time_ms).unwrap_or(0.0),
        )),
    ]
}

fn pressure_focus_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let memory_pressure = snapshot.computed.memory_pressure * 100.0;
    let disk_sleep = snapshot
        .processes
        .iter()
        .filter(|proc| proc.state == ProcessState::DiskSleep)
        .count();
    let cgroup_mem = snapshot
        .linux
        .as_ref()
        .and_then(|linux| linux.cgroup.as_ref())
        .map(|cgroup| cgroup.memory_usage_pct)
        .unwrap_or(0.0);

    lines.push(Line::from(vec![
        Span::styled(
            text(locale, "lecture:", "reading:"),
            theme.highlight_style(),
        ),
        Span::raw(" "),
        Span::raw(if memory_pressure >= 85.0 && cgroup_mem >= 85.0 {
            text(
                locale,
                "pression memoire visible a la fois cote hote et cgroup",
                "memory pressure is visible both at host and cgroup level",
            )
        } else if memory_pressure >= 85.0 {
            text(
                locale,
                "pression memoire plutot host-wide",
                "memory pressure looks more host-wide",
            )
        } else {
            text(
                locale,
                "pression memoire encore contenue",
                "memory pressure is still contained",
            )
        }),
    ]));

    lines.push(Line::from(if disk_sleep > 0 {
        text(
            locale,
            "des processus en etat D indiquent une attente IO ou reclaim severe",
            "disk-sleep processes point to IO wait or severe reclaim stalls",
        )
    } else {
        text(
            locale,
            "pas de file visible en etat D dans le top courant",
            "no visible D-state queue in the current top slice",
        )
    }));

    lines
}

fn network_focus_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let total_errors = snapshot
        .networks
        .iter()
        .map(|net| net.rx_errors + net.tx_errors + net.rx_dropped + net.tx_dropped)
        .sum::<u64>();
    let retrans = snapshot
        .networks
        .first()
        .map(|net| net.retrans_segs)
        .unwrap_or(0);
    let hottest = snapshot
        .networks
        .iter()
        .max_by_key(|net| net.rx_bytes_sec + net.tx_bytes_sec);

    vec![
        Line::from(vec![
            Span::styled(
                text(locale, "hot iface:", "hot iface:"),
                theme.highlight_style(),
            ),
            Span::raw(" "),
            Span::raw(
                hottest
                    .map(|net| net.interface.clone())
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]),
        Line::from(if retrans > 0 {
            text(
                locale,
                "des retransmissions existent: verifier perte, congestion ou cible lente",
                "retransmissions are present: check loss, congestion, or a slow peer",
            )
        } else {
            text(
                locale,
                "pas de retrans visible sur ce snapshot",
                "no visible retransmissions on this snapshot",
            )
        }),
        Line::from(if total_errors > 0 {
            text(
                locale,
                "erreurs ou drops visibles: suspecter lien, pilote ou saturation locale",
                "errors or drops are visible: suspect link, driver, or local saturation",
            )
        } else {
            text(
                locale,
                "aucun drop/erreur visible: lecture plus orientee socket que lien",
                "no visible errors/drops: this readout is more socket-driven than link-driven",
            )
        }),
    ]
}

fn jvm_focus_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let jvms: Vec<&ProcessMetrics> = snapshot
        .processes
        .iter()
        .filter(|proc| proc.is_jvm)
        .collect();
    let top_threads = jvms.iter().copied().max_by_key(|proc| proc.threads);
    let top_cpu = jvms
        .iter()
        .copied()
        .max_by(|a, b| a.cpu_pct.partial_cmp(&b.cpu_pct).unwrap_or(Ordering::Equal));

    vec![
        Line::from(vec![
            Span::styled(
                text(locale, "top cpu:", "top cpu:"),
                theme.highlight_style(),
            ),
            Span::raw(" "),
            Span::raw(
                top_cpu
                    .map(|proc| format!("{} {:.1}%", proc.name, proc.cpu_pct))
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                text(locale, "top threads:", "top threads:"),
                theme.highlight_style(),
            ),
            Span::raw(" "),
            Span::raw(
                top_threads
                    .map(|proc| format!("{} {}", proc.name, proc.threads))
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]),
        Line::from(if jvms.is_empty() {
            text(
                locale,
                "aucune JVM detectee dans le top courant",
                "no detected JVM in the current top slice",
            )
        } else {
            text(
                locale,
                "la vue reste locale et heuristique, avant des outils JVM plus intrusifs",
                "this view stays local and heuristic before using more intrusive JVM tools",
            )
        }),
    ]
}

fn disk_focus_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let hottest_disk = snapshot.disks.iter().max_by(|a, b| {
        a.util_pct
            .partial_cmp(&b.util_pct)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                a.await_ms
                    .partial_cmp(&b.await_ms)
                    .unwrap_or(Ordering::Equal)
            })
    });
    let d_state = snapshot
        .processes
        .iter()
        .filter(|proc| proc.state == ProcessState::DiskSleep)
        .count();

    vec![
        Line::from(vec![
            Span::styled(
                text(locale, "hot disk:", "hot disk:"),
                theme.highlight_style(),
            ),
            Span::raw(" "),
            Span::raw(
                hottest_disk
                    .map(|disk| {
                        format!(
                            "{} util {:.1}% await {:.1}ms",
                            disk.device, disk.util_pct, disk.await_ms
                        )
                    })
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]),
        Line::from(if d_state > 0 {
            text(
                locale,
                "des processus attendent deja le blocage disque",
                "some processes are already waiting on storage blocking",
            )
        } else {
            text(
                locale,
                "pas de D-state visible: verifier plutot debit et queue depth",
                "no visible D-state: focus on throughput and queue depth instead",
            )
        }),
        Line::from(
            if hottest_disk.is_some_and(|disk| disk.await_ms >= 20.0 || disk.queue_depth >= 1.0) {
                text(
                    locale,
                    "latence et file d'attente pointent vers une contention reelle",
                    "latency and queue depth point to real contention",
                )
            } else {
                text(
                    locale,
                    "activite disque visible mais contention encore moderee",
                    "disk activity is visible but contention is still moderate",
                )
            },
        ),
    ]
}

fn metric_table<'a, I>(
    title: &'static str,
    header_cells: Vec<Cell<'a>>,
    rows: Vec<Row<'a>>,
    widths: I,
    theme: &Theme,
) -> Table<'a>
where
    I: IntoIterator<Item = Constraint>,
{
    Table::new(rows, widths)
        .header(Row::new(header_cells).style(theme.highlight_style()))
        .block(
            Block::default()
                .title(Line::from(vec![Span::styled(title, theme.title_style())]))
                .borders(Borders::ALL)
                .border_style(theme.border_style()),
        )
}

fn key_value_row<'a>(key: &'a str, value: String, value_style: Style) -> Row<'a> {
    Row::new(vec![Cell::from(key), Cell::from(value).style(value_style)])
}

fn process_pressure_score(proc: &ProcessMetrics) -> u64 {
    let disk_sleep_weight = if proc.state == ProcessState::DiskSleep {
        1_000_000
    } else {
        0
    };
    disk_sleep_weight + proc.threads as u64 * 1_000 + proc.mem_rss_kb / 1024 + proc.cpu_pct as u64
}

fn severity_style(value: f64, critical: f64, warning: f64, theme: &Theme) -> Style {
    if value >= critical {
        theme.alert_style()
    } else if value >= warning {
        theme.highlight_style()
    } else {
        body_style(theme)
    }
}

fn body_style(theme: &Theme) -> Style {
    Style::default().fg(theme.text)
}

fn style_for_pressure(value: f64, theme: &Theme) -> Style {
    if value >= 90.0 {
        theme.alert_style()
    } else if value >= 75.0 {
        theme.highlight_style()
    } else {
        theme.muted_style()
    }
}

fn kb_to_gb(value_kb: u64) -> f64 {
    value_kb as f64 / (1024.0 * 1024.0)
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn jvm_role(proc: &ProcessMetrics, locale: Locale) -> &'static str {
    let cmd = proc.cmdline.to_ascii_lowercase();
    if cmd.contains("spring") || cmd.contains("boot") {
        text(locale, "service", "service")
    } else if cmd.contains("kafka") {
        "kafka"
    } else if cmd.contains("tomcat") || cmd.contains("jetty") {
        "web"
    } else if cmd.contains(".jar") {
        "jar"
    } else if cmd.contains("gradle") || cmd.contains("maven") {
        text(locale, "build", "build")
    } else {
        text(locale, "jvm", "jvm")
    }
}

fn jvm_dominant_pressure(proc: &ProcessMetrics, locale: Locale) -> &'static str {
    let io_mb = (proc.io_read_bytes + proc.io_write_bytes) as f64 / (1024.0 * 1024.0);
    let mut dominant = ("cpu", proc.cpu_pct);
    for candidate in [
        ("rss", proc.mem_rss_kb as f64 / 1024.0),
        ("threads", proc.threads as f64),
        ("fds", proc.fd_count as f64),
        ("io", io_mb),
    ] {
        if candidate.1 > dominant.1 {
            dominant = candidate;
        }
    }
    match dominant.0 {
        "rss" => text(locale, "memoire", "memory"),
        "threads" => text(locale, "threads", "threads"),
        "fds" => "fds",
        "io" => "io",
        _ => "cpu",
    }
}

fn jvm_heap_hint(proc: &ProcessMetrics) -> String {
    extract_cmd_flag(&proc.cmdline, "-Xmx").unwrap_or_else(|| "-".to_string())
}

fn extract_cmd_flag(cmdline: &str, flag: &str) -> Option<String> {
    cmdline
        .split_whitespace()
        .find(|part| part.starts_with(flag))
        .map(|part| part.to_string())
}
