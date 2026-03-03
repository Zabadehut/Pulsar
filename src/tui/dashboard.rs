use crate::collectors::Snapshot;
use crate::reference::{self, Locale, SearchHit, UiVisibility};
use crate::tui::{
    i18n::text,
    theme::Theme,
    widgets::{
        alerts_widget, cpu_widget, disk_widget, linux_widget, memory_widget, network_widget,
        process_widget, reference_widget, system_widget,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(Debug, Clone, Copy)]
pub enum Panel {
    System,
    Cpu,
    Memory,
    Linux,
    Disk,
    Network,
    Alerts,
    Process,
}

#[derive(Debug, Clone, Copy)]
struct PanelVisibility {
    system: bool,
    cpu: bool,
    memory: bool,
    linux: bool,
    disk: bool,
    network: bool,
    alerts: bool,
    process: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            system: true,
            cpu: true,
            memory: true,
            linux: true,
            disk: true,
            network: true,
            alerts: true,
            process: true,
        }
    }
}

impl PanelVisibility {
    fn toggle(&mut self, panel: Panel) {
        match panel {
            Panel::System => self.system = !self.system,
            Panel::Cpu => self.cpu = !self.cpu,
            Panel::Memory => self.memory = !self.memory,
            Panel::Linux => self.linux = !self.linux,
            Panel::Disk => self.disk = !self.disk,
            Panel::Network => self.network = !self.network,
            Panel::Alerts => self.alerts = !self.alerts,
            Panel::Process => self.process = !self.process,
        }
    }

    fn is_visible(&self, panel: Panel) -> bool {
        match panel {
            Panel::System => self.system,
            Panel::Cpu => self.cpu,
            Panel::Memory => self.memory,
            Panel::Linux => self.linux,
            Panel::Disk => self.disk,
            Panel::Network => self.network,
            Panel::Alerts => self.alerts,
            Panel::Process => self.process,
        }
    }

    fn visible_count(&self) -> usize {
        [
            self.system,
            self.cpu,
            self.memory,
            self.linux,
            self.disk,
            self.network,
            self.alerts,
            self.process,
        ]
        .into_iter()
        .filter(|visible| *visible)
        .count()
    }
}

pub struct Dashboard {
    pub theme_name: String,
    pub theme: Theme,
    locale: Locale,
    visibility: PanelVisibility,
    operator_mode: OperatorMode,
    detail_level: DetailLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum OperatorMode {
    Overview,
    Storage,
    Network,
    Process,
    Pressure,
    Full,
}

#[derive(Debug, Clone, Copy)]
pub enum DetailLevel {
    Compact,
    Detailed,
}

#[derive(Debug, Clone, Default)]
pub struct ReferenceUiState {
    pub visible: bool,
    pub input_active: bool,
    pub query: String,
    pub selected: usize,
}

impl Dashboard {
    pub fn new(theme_name: &str, locale: Locale) -> Self {
        let theme_name = Theme::normalize_name(theme_name).to_string();
        Self {
            theme: Theme::from_name(&theme_name),
            theme_name,
            locale,
            visibility: PanelVisibility::default(),
            operator_mode: OperatorMode::Full,
            detail_level: DetailLevel::Detailed,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme_name = Theme::next_name(&self.theme_name).to_string();
        self.theme = Theme::from_name(&self.theme_name);
    }

    pub fn toggle_panel(&mut self, panel: Panel) {
        self.visibility.toggle(panel);
    }

    pub fn cycle_locale(&mut self) {
        self.locale = self.locale.next();
    }

    pub fn toggle_detail(&mut self) {
        self.detail_level = match self.detail_level {
            DetailLevel::Compact => DetailLevel::Detailed,
            DetailLevel::Detailed => DetailLevel::Compact,
        };
    }

    pub fn set_operator_mode(&mut self, mode: OperatorMode) {
        self.operator_mode = mode;
        self.visibility = mode.visibility();
    }

    pub fn render(&self, frame: &mut Frame, snapshot: &Snapshot, reference: &ReferenceUiState) {
        let area = frame.area();

        // Layout principal : header + corps + footer
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // header
                Constraint::Min(0),    // corps
                Constraint::Length(1), // footer
            ])
            .split(area);

        self.render_header(frame, main[0], snapshot, reference);
        self.render_body(frame, main[1], snapshot, reference);
        self.render_footer(frame, main[2], snapshot, reference);
    }

    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
    ) {
        let hostname = snapshot
            .system
            .as_ref()
            .map(|s| s.hostname.as_str())
            .unwrap_or("unknown");

        let uptime = snapshot
            .system
            .as_ref()
            .map(|s| format_uptime(s.uptime_seconds))
            .unwrap_or_default();

        let os = snapshot
            .system
            .as_ref()
            .map(|s| s.os_name.clone())
            .unwrap_or_default();

        let ts = chrono::DateTime::from_timestamp(snapshot.timestamp, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_default();

        let header = Paragraph::new(Line::from(vec![
            Span::styled(" ◉ PULSAR ", self.theme.title_style()),
            Span::raw(format!("  {}  {}  {}  {}", hostname, os, uptime, ts)),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{}:{}",
                    text(self.locale, "mode", "mode"),
                    self.operator_mode.label(self.locale)
                ),
                self.theme.highlight_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{}:{}",
                    text(self.locale, "lang", "lang"),
                    self.locale.code()
                ),
                self.theme.highlight_style(),
            ),
            Span::raw("  "),
            Span::styled(
                if reference.query.is_empty() {
                    format!(
                        "{}:{}",
                        text(self.locale, "index", "index"),
                        text(self.locale, "off", "off")
                    )
                } else {
                    format!(
                        "{}:{}",
                        text(self.locale, "search", "search"),
                        reference.query
                    )
                },
                if reference.query.is_empty() {
                    self.theme.muted_style()
                } else {
                    self.theme.highlight_style()
                },
            ),
        ]));
        frame.render_widget(header, area);
    }

    fn render_body(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
    ) {
        if reference.visible {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(area);
            self.render_monitoring(frame, cols[0], snapshot, reference);
            self.render_reference(frame, cols[1], reference);
        } else {
            self.render_monitoring(frame, area, snapshot, reference);
        }
    }

    fn render_monitoring(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
    ) {
        let left_panels = [Panel::System, Panel::Cpu, Panel::Memory, Panel::Linux];
        let right_panels = [Panel::Disk, Panel::Network, Panel::Alerts];

        let has_left = left_panels
            .into_iter()
            .any(|panel| self.visibility.is_visible(panel));
        let has_right = right_panels
            .into_iter()
            .any(|panel| self.visibility.is_visible(panel));
        let has_top = has_left || has_right;
        let has_process = self.visibility.is_visible(Panel::Process);

        match (has_top, has_process) {
            (true, true) => {
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(25), Constraint::Min(0)])
                    .split(area);
                self.render_top(frame, rows[0], snapshot, has_left, has_right, reference);
                process_widget::render(
                    frame,
                    rows[1],
                    &snapshot.processes,
                    self.locale,
                    matches!(self.detail_level, DetailLevel::Detailed),
                    &self.theme,
                    self.panel_highlighted(Panel::Process, reference),
                );
            }
            (true, false) => self.render_top(frame, area, snapshot, has_left, has_right, reference),
            (false, true) => process_widget::render(
                frame,
                area,
                &snapshot.processes,
                self.locale,
                matches!(self.detail_level, DetailLevel::Detailed),
                &self.theme,
                self.panel_highlighted(Panel::Process, reference),
            ),
            (false, false) => {
                frame.render_widget(
                    Paragraph::new(text(
                        self.locale,
                        "Tous les panneaux sont caches. Basculer avec s/c/m/l/d/n/a/p.",
                        "All panels hidden. Toggle with s/c/m/l/d/n/a/p.",
                    )),
                    area,
                );
            }
        }
    }

    fn render_top(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        has_left: bool,
        has_right: bool,
        reference: &ReferenceUiState,
    ) {
        match (has_left, has_right) {
            (true, true) => {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(area);
                self.render_left_stack(frame, cols[0], snapshot, reference);
                self.render_right_stack(frame, cols[1], snapshot, reference);
            }
            (true, false) => self.render_left_stack(frame, area, snapshot, reference),
            (false, true) => self.render_right_stack(frame, area, snapshot, reference),
            (false, false) => {}
        }
    }

    fn render_left_stack(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
    ) {
        let panels = self.visible_panels(&[Panel::System, Panel::Cpu, Panel::Memory, Panel::Linux]);
        let chunks = split_vertical(area, panels.len());

        for (panel, chunk) in panels.into_iter().zip(chunks.into_iter()) {
            match panel {
                Panel::System => system_widget::render(
                    frame,
                    chunk,
                    snapshot.system.as_ref(),
                    self.locale,
                    matches!(self.detail_level, DetailLevel::Detailed),
                    &self.theme,
                    self.panel_highlighted(Panel::System, reference),
                ),
                Panel::Cpu => cpu_widget::render(
                    frame,
                    chunk,
                    cpu_widget::CpuWidgetState {
                        metrics: snapshot.cpu.as_ref(),
                        trend_p50: snapshot.computed.cpu_trend_p50,
                        trend_p95: snapshot.computed.cpu_trend_p95,
                        locale: self.locale,
                        detailed: matches!(self.detail_level, DetailLevel::Detailed),
                        highlighted: self.panel_highlighted(Panel::Cpu, reference),
                    },
                    &self.theme,
                ),
                Panel::Memory => memory_widget::render(
                    frame,
                    chunk,
                    memory_widget::MemoryWidgetState {
                        metrics: snapshot.memory.as_ref(),
                        memory_pressure: snapshot.computed.memory_pressure,
                        locale: self.locale,
                        detailed: matches!(self.detail_level, DetailLevel::Detailed),
                        highlighted: self.panel_highlighted(Panel::Memory, reference),
                    },
                    &self.theme,
                ),
                Panel::Linux => linux_widget::render(
                    frame,
                    chunk,
                    snapshot.linux.as_ref(),
                    self.locale,
                    matches!(self.detail_level, DetailLevel::Detailed),
                    &self.theme,
                    self.panel_highlighted(Panel::Linux, reference),
                ),
                _ => {}
            }
        }
    }

    fn render_right_stack(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
    ) {
        let panels = self.visible_panels(&[Panel::Disk, Panel::Network, Panel::Alerts]);
        let chunks = split_vertical(area, panels.len());

        for (panel, chunk) in panels.into_iter().zip(chunks.into_iter()) {
            match panel {
                Panel::Disk => disk_widget::render(
                    frame,
                    chunk,
                    &snapshot.disks,
                    self.locale,
                    matches!(self.detail_level, DetailLevel::Detailed),
                    &self.theme,
                    self.panel_highlighted(Panel::Disk, reference),
                ),
                Panel::Network => network_widget::render(
                    frame,
                    chunk,
                    &snapshot.networks,
                    self.locale,
                    matches!(self.detail_level, DetailLevel::Detailed),
                    &self.theme,
                    self.panel_highlighted(Panel::Network, reference),
                ),
                Panel::Alerts => alerts_widget::render(
                    frame,
                    chunk,
                    &snapshot.computed.alerts,
                    self.locale,
                    &self.theme,
                    self.panel_highlighted(Panel::Alerts, reference),
                ),
                _ => {}
            }
        }
    }

    fn render_reference(&self, frame: &mut Frame, area: Rect, reference: &ReferenceUiState) {
        let hits = self.reference_hits(reference);
        let selected = reference.selected.min(hits.len().saturating_sub(1));
        let visible_count = hits
            .iter()
            .filter(|hit| hit.entry.ui_visibility == UiVisibility::Visible)
            .count();
        reference_widget::render(
            frame,
            area,
            reference_widget::ReferenceWidgetState {
                query: &reference.query,
                mode: self.operator_mode.label(self.locale),
                locale: self.locale,
                visible_count,
                indexed_only_count: hits.len().saturating_sub(visible_count),
                hits: &hits,
                selected,
            },
            &self.theme,
        );
    }

    fn reference_hits(&self, reference: &ReferenceUiState) -> Vec<SearchHit> {
        if reference.query.is_empty() {
            let mut hits: Vec<SearchHit> = reference::catalog_views(self.locale)
                .into_iter()
                .enumerate()
                .map(|(index, entry)| SearchHit {
                    score: self
                        .operator_mode
                        .reference_bias(entry.panel, entry.category, index),
                    entry,
                })
                .collect();
            hits.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then_with(|| a.entry.title.cmp(b.entry.title))
            });
            hits
        } else {
            reference::search(&reference.query, self.locale)
        }
    }

    fn panel_highlighted(&self, panel: Panel, reference: &ReferenceUiState) -> bool {
        reference::panel_matches_query(self.panel_key(panel), &reference.query)
    }

    fn panel_key(&self, panel: Panel) -> &'static str {
        match panel {
            Panel::System => "system",
            Panel::Cpu => "cpu",
            Panel::Memory => "memory",
            Panel::Linux => "linux",
            Panel::Disk => "disk",
            Panel::Network => "network",
            Panel::Alerts => "alerts",
            Panel::Process => "process",
        }
    }

    fn visible_panels(&self, panels: &[Panel]) -> Vec<Panel> {
        panels
            .iter()
            .copied()
            .filter(|panel| self.visibility.is_visible(*panel))
            .collect()
    }

    fn render_footer(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
    ) {
        let alerts = snapshot.computed.alerts.len();
        let visibility = &self.visibility;
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" q", self.theme.highlight_style()),
            Span::raw(":quit  "),
            Span::styled("r", self.theme.highlight_style()),
            Span::raw(":refresh  "),
            Span::styled("t", self.theme.highlight_style()),
            Span::raw(format!(":theme({})  ", self.theme_name)),
            Span::styled("i", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}({})  ",
                text(self.locale, "lang", "lang"),
                self.locale.code()
            )),
            Span::styled("v", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}({})  ",
                text(self.locale, "detail", "detail"),
                self.detail_level.label(self.locale)
            )),
            Span::styled("/", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "search", "search"))),
            Span::styled("?", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "index", "index"))),
            Span::styled("esc", self.theme.highlight_style()),
            Span::raw(if reference.input_active {
                text(self.locale, ":fermer recherche  ", ":close search  ")
            } else if reference.visible {
                text(self.locale, ":fermer index  ", ":close index  ")
            } else {
                text(self.locale, ":vider  ", ":clear  ")
            }),
            Span::styled("1", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Overview.label(self.locale))),
            Span::styled("2", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Storage.label(self.locale))),
            Span::styled("3", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Network.label(self.locale))),
            Span::styled("4", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Process.label(self.locale))),
            Span::styled("5", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Pressure.label(self.locale))),
            Span::styled("6", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Full.label(self.locale))),
            panel_toggle_span(
                "s",
                text(self.locale, "sys", "sys"),
                visibility.system,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "c",
                text(self.locale, "cpu", "cpu"),
                visibility.cpu,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "m",
                text(self.locale, "mem", "mem"),
                visibility.memory,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "l",
                text(self.locale, "linux", "linux"),
                visibility.linux,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "d",
                text(self.locale, "disk", "disk"),
                visibility.disk,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "n",
                text(self.locale, "net", "net"),
                visibility.network,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "a",
                text(self.locale, "alertes", "alerts"),
                visibility.alerts,
                &self.theme,
            ),
            Span::raw(" "),
            panel_toggle_span(
                "p",
                text(self.locale, "proc", "proc"),
                visibility.process,
                &self.theme,
            ),
            Span::raw(format!(
                "  {}:{}/8  ",
                text(self.locale, "visibles", "visible"),
                self.visibility.visible_count()
            )),
            Span::styled(
                format!(
                    "{}:{} w:{} c:{}  ",
                    text(self.locale, "alertes", "alerts"),
                    alerts,
                    snapshot.computed.alerts_warning,
                    snapshot.computed.alerts_critical
                ),
                if alerts > 0 {
                    self.theme.alert_style()
                } else {
                    self.theme.highlight_style()
                },
            ),
            Span::raw("  "),
            Span::styled(
                if reference.query.is_empty() {
                    format!(
                        "{}:{}",
                        text(self.locale, "reference", "reference"),
                        text(self.locale, "tout", "all")
                    )
                } else {
                    format!(
                        "{}:{}",
                        text(self.locale, "reference", "reference"),
                        reference.query
                    )
                },
                if reference.query.is_empty() {
                    self.theme.muted_style()
                } else {
                    self.theme.highlight_style()
                },
            ),
            Span::raw("  Pulsar v0.1.0 — Kevin Vanden-Brande"),
        ]));
        frame.render_widget(footer, area);
    }
}

fn split_vertical(area: Rect, count: usize) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![area];
    }

    let constraints = vec![Constraint::Ratio(1, count as u32); count];
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .iter()
        .copied()
        .collect()
}

fn panel_toggle_span<'a>(key: &'a str, label: &'a str, visible: bool, theme: &Theme) -> Span<'a> {
    let text = if visible {
        format!("{key}:{label}")
    } else {
        format!("{key}:{label} off")
    };

    Span::styled(
        text,
        if visible {
            theme.highlight_style()
        } else {
            theme.muted_style()
        },
    )
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("up {}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("up {}h {}m", hours, mins)
    } else {
        format!("up {}m", mins)
    }
}

impl OperatorMode {
    pub fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Overview => text(locale, "vue", "overview"),
            Self::Storage => text(locale, "stockage", "storage"),
            Self::Network => text(locale, "reseau", "network"),
            Self::Process => text(locale, "processus", "process"),
            Self::Pressure => text(locale, "pression", "pressure"),
            Self::Full => text(locale, "complet", "full"),
        }
    }

    fn visibility(self) -> PanelVisibility {
        match self {
            Self::Overview => PanelVisibility {
                system: true,
                cpu: true,
                memory: true,
                linux: false,
                disk: false,
                network: false,
                alerts: true,
                process: true,
            },
            Self::Storage => PanelVisibility {
                system: false,
                cpu: false,
                memory: true,
                linux: true,
                disk: true,
                network: false,
                alerts: true,
                process: true,
            },
            Self::Network => PanelVisibility {
                system: true,
                cpu: true,
                memory: false,
                linux: true,
                disk: false,
                network: true,
                alerts: true,
                process: false,
            },
            Self::Process => PanelVisibility {
                system: true,
                cpu: true,
                memory: true,
                linux: false,
                disk: false,
                network: false,
                alerts: true,
                process: true,
            },
            Self::Pressure => PanelVisibility {
                system: false,
                cpu: true,
                memory: true,
                linux: true,
                disk: true,
                network: false,
                alerts: true,
                process: false,
            },
            Self::Full => PanelVisibility::default(),
        }
    }

    fn reference_bias(self, panel: &str, category: &str, index: usize) -> usize {
        let preferred = match self {
            Self::Overview => matches!(panel, "system" | "cpu" | "memory" | "alerts" | "process"),
            Self::Storage => matches!(panel, "disk" | "linux" | "memory" | "alerts"),
            Self::Network => matches!(panel, "network" | "system" | "cpu" | "linux"),
            Self::Process => matches!(panel, "process" | "cpu" | "memory" | "alerts"),
            Self::Pressure => {
                matches!(panel, "memory" | "linux" | "alerts" | "disk" | "cpu")
                    || matches!(category, "memory" | "linux" | "disk")
            }
            Self::Full => true,
        };

        if preferred {
            10_000usize.saturating_sub(index)
        } else {
            1_000usize.saturating_sub(index)
        }
    }
}

impl DetailLevel {
    fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Compact => text(locale, "compact", "compact"),
            Self::Detailed => text(locale, "detail", "detailed"),
        }
    }
}
