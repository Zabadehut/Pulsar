use crate::collectors::{process::ProcessState, Snapshot};
use crate::reference::Locale;
use crate::tui::{i18n::text, theme::Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

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

pub fn render(
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
        SpecialistView::Pressure => pressure_lines(snapshot, locale, theme),
        SpecialistView::Network => network_lines(snapshot, locale, theme),
        SpecialistView::Jvm => jvm_lines(snapshot, locale, theme),
        SpecialistView::DiskPressure => disk_pressure_lines(snapshot, locale, theme),
        SpecialistView::None => Vec::new(),
    };

    frame.render_widget(Paragraph::new(lines), inner);
}

fn pressure_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
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

fn network_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
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

fn jvm_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
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
    let top_cpu = jvms.iter().max_by(|a, b| {
        a.cpu_pct
            .partial_cmp(&b.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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

fn disk_pressure_lines(snapshot: &Snapshot, locale: Locale, theme: &Theme) -> Vec<Line<'static>> {
    let hottest_disk = snapshot.disks.iter().max_by(|a, b| {
        a.util_pct
            .partial_cmp(&b.util_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.await_ms
                    .partial_cmp(&b.await_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
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

fn style_for_pressure(value: f64, theme: &Theme) -> ratatui::style::Style {
    if value >= 90.0 {
        theme.alert_style()
    } else if value >= 75.0 {
        theme.highlight_style()
    } else {
        theme.muted_style()
    }
}
