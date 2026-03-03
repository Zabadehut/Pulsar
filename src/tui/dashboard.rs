use crate::collectors::{LogEntry, Snapshot};
use crate::log_sources;
use crate::reference::{self, Locale, SearchHit, UiVisibility};
use crate::tui::{
    i18n::text,
    theme::Theme,
    widgets::{
        alerts_widget,
        analysis_widget::{self, SpecialistView},
        cpu_widget, disk_widget, linux_widget, log_widget, memory_widget, network_widget,
        process_widget, reference_widget, system_widget,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::collections::HashMap;

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
    specialist_view: SpecialistView,
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

#[derive(Debug, Clone, Default)]
pub struct LogUiState {
    pub visible: bool,
    pub input_active: bool,
    pub query: String,
    pub targets: Vec<String>,
    pub entries: Vec<LogEntry>,
    pub active_files: Vec<String>,
    pub file_states: HashMap<String, log_sources::FileTailState>,
    pub errors_only: bool,
    pub rotated_files: usize,
    pub last_error: Option<String>,
    pub last_refresh_ts: i64,
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
            specialist_view: SpecialistView::None,
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
        self.specialist_view = SpecialistView::None;
        self.operator_mode = mode;
        self.visibility = mode.visibility();
    }

    pub fn set_specialist_view(&mut self, specialist: SpecialistView) {
        self.specialist_view = specialist;
        self.operator_mode = match specialist {
            SpecialistView::None => self.operator_mode,
            SpecialistView::Pressure => OperatorMode::Pressure,
            SpecialistView::Network => OperatorMode::Network,
            SpecialistView::Jvm => OperatorMode::Process,
            SpecialistView::DiskPressure => OperatorMode::Storage,
            SpecialistView::DiskInventory => OperatorMode::Storage,
        };
        if specialist != SpecialistView::None {
            self.visibility = self.operator_mode.visibility();
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
        logs: &LogUiState,
    ) {
        let area = frame.area();
        let header_height = self.header_height(area.width);
        let footer_height = self.footer_height(snapshot, reference, logs, area.width);

        // Layout principal : header + corps + footer
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height), // header
                Constraint::Min(0),                // corps
                Constraint::Length(footer_height), // footer
            ])
            .split(area);

        self.render_header(frame, main[0], snapshot, reference, logs);
        self.render_body(frame, main[1], snapshot, reference, logs);
        self.render_footer(frame, main[2], snapshot, reference, logs);
    }

    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
        logs: &LogUiState,
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

        let header = Paragraph::new(
            self.header_lines(hostname, &os, &uptime, &ts, reference, logs, area.width),
        );
        frame.render_widget(header, area);
    }

    fn render_body(
        &self,
        frame: &mut Frame,
        area: Rect,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
        logs: &LogUiState,
    ) {
        if reference.visible {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(area);
            self.render_monitoring(frame, cols[0], snapshot, reference);
            self.render_reference(frame, cols[1], reference);
        } else if logs.visible {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
                .split(area);
            self.render_monitoring(frame, cols[0], snapshot, reference);
            self.render_logs(frame, cols[1], snapshot, logs);
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
        if self.specialist_view != SpecialistView::None {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(analysis_widget::summary_height(matches!(
                        self.detail_level,
                        DetailLevel::Detailed
                    ))),
                    Constraint::Min(0),
                ])
                .split(area);
            analysis_widget::render_summary(
                frame,
                rows[0],
                snapshot,
                self.specialist_view,
                self.locale,
                &self.theme,
            );
            analysis_widget::render_drilldown(
                frame,
                rows[1],
                snapshot,
                self.specialist_view,
                self.locale,
                matches!(self.detail_level, DetailLevel::Detailed),
                &self.theme,
            );
            return;
        }

        let monitoring_area = area;

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
                    .split(monitoring_area);
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
            (true, false) => self.render_top(
                frame,
                monitoring_area,
                snapshot,
                has_left,
                has_right,
                reference,
            ),
            (false, true) => process_widget::render(
                frame,
                monitoring_area,
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
                        "Tous les panneaux sont caches. Basculer avec s/c/m/k/d/n/a/p.",
                        "All panels hidden. Toggle with s/c/m/k/d/n/a/p.",
                    )),
                    monitoring_area,
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
                    snapshot
                        .logs
                        .as_ref()
                        .map(|logs| logs.system_events.as_slice())
                        .unwrap_or(&[]),
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
                mode: self.reference_context_label(),
                locale: self.locale,
                visible_count,
                indexed_only_count: hits.len().saturating_sub(visible_count),
                hits: &hits,
                selected,
            },
            &self.theme,
        );
    }

    fn render_logs(&self, frame: &mut Frame, area: Rect, snapshot: &Snapshot, logs: &LogUiState) {
        let mut entries = snapshot
            .logs
            .as_ref()
            .map(|item| item.system_events.clone())
            .unwrap_or_default();
        entries.extend(logs.entries.clone());
        entries.sort_by(|a, b| {
            b.timestamp
                .cmp(&a.timestamp)
                .then_with(|| a.origin.cmp(&b.origin))
        });
        let filtered_entries = entries
            .into_iter()
            .filter(|entry| {
                !logs.errors_only
                    || matches!(
                        entry.level,
                        crate::collectors::AlertLevel::Critical
                            | crate::collectors::AlertLevel::Warning
                    )
            })
            .collect::<Vec<_>>();
        log_widget::render(
            frame,
            area,
            log_widget::LogWidgetState {
                locale: self.locale,
                targets: &logs.targets,
                active_files: &logs.active_files,
                query: &logs.query,
                input_active: logs.input_active,
                errors_only: logs.errors_only,
                rotated_files: logs.rotated_files,
                entries: &filtered_entries,
                error: logs.last_error.as_deref(),
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
                        .reference_bias(entry.panel, entry.category, index)
                        + self.specialist_view.reference_bias(
                            entry.panel,
                            entry.category,
                            entry.id,
                            index,
                        ),
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
        logs: &LogUiState,
    ) {
        let alerts = snapshot.computed.alerts.len();
        let visibility = &self.visibility;
        let footer = Paragraph::new(
            self.footer_lines(alerts, visibility, snapshot, reference, logs, area.width),
        );
        frame.render_widget(footer, area);
    }

    fn footer_height(
        &self,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
        logs: &LogUiState,
        width: u16,
    ) -> u16 {
        self.footer_lines(
            snapshot.computed.alerts.len(),
            &self.visibility,
            snapshot,
            reference,
            logs,
            width,
        )
        .len() as u16
    }

    fn header_height(&self, width: u16) -> u16 {
        if width >= 150 {
            1
        } else if width >= 95 {
            2
        } else {
            3
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn header_lines(
        &self,
        hostname: &str,
        os: &str,
        uptime: &str,
        ts: &str,
        reference: &ReferenceUiState,
        logs: &LogUiState,
        width: u16,
    ) -> Vec<Line<'static>> {
        let title = Span::styled(" ◉ PULSAR ", self.theme.title_style());
        let host_line = Line::from(vec![
            title.clone(),
            Span::raw(format!("  {hostname}  {os}  {uptime}  {ts}")),
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
            self.logs_header_span(logs),
            Span::raw("  "),
            self.reference_header_span(reference),
        ]);

        let system_line = Line::from(vec![
            title.clone(),
            Span::raw(format!("  {hostname}")),
            Span::raw("  "),
            Span::styled(os.to_string(), self.theme.muted_style()),
            Span::raw("  "),
            Span::styled(uptime.to_string(), self.theme.muted_style()),
            Span::raw("  "),
            Span::styled(ts.to_string(), self.theme.highlight_style()),
        ]);

        let context_line = Line::from(vec![
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
            self.logs_header_span(logs),
            Span::raw("  "),
            self.reference_header_span(reference),
        ]);

        let identity_line = Line::from(vec![
            title,
            Span::raw(format!("  {hostname}")),
            Span::raw("  "),
            Span::styled(ts.to_string(), self.theme.highlight_style()),
        ]);

        if width >= 150 {
            vec![host_line]
        } else if width >= 95 {
            vec![system_line, context_line]
        } else {
            vec![
                identity_line,
                Line::from(vec![
                    Span::styled(os.to_string(), self.theme.muted_style()),
                    Span::raw("  "),
                    Span::styled(uptime.to_string(), self.theme.muted_style()),
                ]),
                context_line,
            ]
        }
    }

    fn reference_header_span(&self, reference: &ReferenceUiState) -> Span<'static> {
        Span::styled(
            if !reference.query.is_empty() {
                format!(
                    "{}:{}",
                    text(self.locale, "search", "search"),
                    reference.query
                )
            } else if reference.visible {
                format!(
                    "{}:{}",
                    text(self.locale, "index", "index"),
                    text(self.locale, "on", "on")
                )
            } else {
                format!(
                    "{}:{}",
                    text(self.locale, "index", "index"),
                    text(self.locale, "off", "off")
                )
            },
            if !reference.query.is_empty() || reference.visible {
                self.theme.highlight_style()
            } else {
                self.theme.muted_style()
            },
        )
    }

    fn logs_header_span(&self, logs: &LogUiState) -> Span<'static> {
        Span::styled(
            if logs.input_active {
                format!(
                    "{}:{}",
                    text(self.locale, "logs", "logs"),
                    text(self.locale, "saisie", "input")
                )
            } else if logs.visible {
                format!(
                    "{}:{}",
                    text(self.locale, "logs", "logs"),
                    text(self.locale, "on", "on")
                )
            } else {
                format!(
                    "{}:{}",
                    text(self.locale, "logs", "logs"),
                    text(self.locale, "off", "off")
                )
            },
            if logs.visible || logs.input_active {
                self.theme.highlight_style()
            } else {
                self.theme.muted_style()
            },
        )
    }

    fn footer_lines(
        &self,
        alerts: usize,
        visibility: &PanelVisibility,
        snapshot: &Snapshot,
        reference: &ReferenceUiState,
        logs: &LogUiState,
        width: u16,
    ) -> Vec<Line<'static>> {
        let nav = Line::from(vec![
            hotkey_span("q", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "quitter", "quit"))),
            hotkey_span("r", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "refresh", "refresh"))),
            hotkey_span("t", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}({})  ",
                text(self.locale, "theme", "theme"),
                self.theme_name
            )),
            hotkey_span("i", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}({})  ",
                text(self.locale, "lang", "lang"),
                self.locale.code()
            )),
            hotkey_span("v", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}({})  ",
                text(self.locale, "detail", "detail"),
                self.detail_level.label(self.locale)
            )),
            hotkey_span("l", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "logs", "logs"))),
            hotkey_span("/", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "search", "search"))),
            hotkey_span("?", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "index", "index"))),
            hotkey_span("esc", self.theme.highlight_style()),
            Span::raw(if logs.input_active {
                text(self.locale, ":fermer saisie logs", ":close log input")
            } else if reference.input_active {
                text(self.locale, ":fermer recherche", ":close search")
            } else if logs.visible {
                text(self.locale, ":fermer logs", ":close logs")
            } else if reference.visible {
                text(self.locale, ":fermer index", ":close index")
            } else {
                text(self.locale, ":vider", ":clear")
            }),
        ]);

        let context = if logs.input_active || logs.visible {
            self.logs_footer_line(logs)
        } else if reference.input_active || reference.visible {
            self.reference_footer_line(reference)
        } else if self.specialist_view != SpecialistView::None {
            self.expert_footer_line()
        } else {
            self.mode_footer_line()
        };

        let panels = self.contextual_panel_line(visibility);
        let status = Line::from(vec![
            Span::styled(
                format!(
                    "{}:{}/8  ",
                    text(self.locale, "visibles", "visible"),
                    self.visibility.visible_count()
                ),
                self.theme.highlight_style(),
            ),
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
            if width >= 140 {
                Span::raw(format!("  Pulsar v{}", env!("CARGO_PKG_VERSION")))
            } else {
                Span::raw("")
            },
        ]);

        let status_compact = width < 110 && self.specialist_view == SpecialistView::None;
        let status = if status_compact {
            self.compact_status_line(alerts, visibility, reference, logs)
        } else {
            status
        };

        if width >= 180 {
            vec![nav, context, merge_line(panels, status)]
        } else {
            vec![nav, context, panels, status]
        }
    }

    fn reference_context_label(&self) -> String {
        if self.specialist_view == SpecialistView::None {
            self.operator_mode.label(self.locale).to_string()
        } else {
            format!(
                "{} + {}",
                self.operator_mode.label(self.locale),
                self.specialist_view.label(self.locale)
            )
        }
    }

    fn mode_footer_line(&self) -> Line<'static> {
        Line::from(vec![
            hotkey_span("1", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Overview.label(self.locale))),
            hotkey_span("2", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Storage.label(self.locale))),
            hotkey_span("3", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Network.label(self.locale))),
            hotkey_span("4", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Process.label(self.locale))),
            hotkey_span("5", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Pressure.label(self.locale))),
            hotkey_span("6", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", OperatorMode::Full.label(self.locale))),
            Span::styled(
                format!(
                    "{}:{}",
                    text(self.locale, "mode", "mode"),
                    self.operator_mode.label(self.locale)
                ),
                self.theme.highlight_style(),
            ),
        ])
    }

    fn expert_footer_line(&self) -> Line<'static> {
        Line::from(vec![
            hotkey_span("7", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                SpecialistView::Pressure.label(self.locale)
            )),
            hotkey_span("8", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", SpecialistView::Network.label(self.locale))),
            hotkey_span("9", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", SpecialistView::Jvm.label(self.locale))),
            hotkey_span("0", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                SpecialistView::DiskPressure.label(self.locale)
            )),
            hotkey_span("g", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                SpecialistView::DiskInventory.label(self.locale)
            )),
            hotkey_span("-", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                text(self.locale, "retour normal", "clear expert")
            )),
            Span::styled(
                format!(
                    "{}:{}",
                    text(self.locale, "expert", "expert"),
                    self.specialist_view.label(self.locale)
                ),
                self.theme.highlight_style(),
            ),
        ])
    }

    fn reference_footer_line(&self, reference: &ReferenceUiState) -> Line<'static> {
        Line::from(vec![
            hotkey_span("/", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "search", "search"))),
            hotkey_span("?", self.theme.highlight_style()),
            Span::raw(format!(":{}  ", text(self.locale, "index", "index"))),
            hotkey_span("up/down", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                text(self.locale, "selection", "selection")
            )),
            hotkey_span("enter", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                text(self.locale, "ouvrir index", "keep index")
            )),
            hotkey_span("esc", self.theme.highlight_style()),
            Span::raw(if reference.input_active {
                text(self.locale, ":fermer recherche", ":close search")
            } else {
                text(self.locale, ":fermer index", ":close index")
            }),
        ])
    }

    fn logs_footer_line(&self, logs: &LogUiState) -> Line<'static> {
        Line::from(vec![
            hotkey_span("l", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                text(self.locale, "ouvrir logs", "open logs")
            )),
            hotkey_span("L", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                text(self.locale, "ajouter chemin", "add path")
            )),
            hotkey_span("e", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}({})  ",
                text(self.locale, "erreurs", "errors"),
                if logs.errors_only {
                    text(self.locale, "on", "on")
                } else {
                    text(self.locale, "off", "off")
                }
            )),
            hotkey_span("enter", self.theme.highlight_style()),
            Span::raw(format!(
                ":{}  ",
                text(self.locale, "valider chemin", "save path")
            )),
            hotkey_span("esc", self.theme.highlight_style()),
            Span::raw(if logs.input_active {
                text(self.locale, ":fermer saisie logs", ":close log input")
            } else {
                text(self.locale, ":fermer logs", ":close logs")
            }),
        ])
    }

    fn contextual_panel_line(&self, visibility: &PanelVisibility) -> Line<'static> {
        let panels = match self.specialist_view {
            SpecialistView::Pressure => &[
                Panel::Cpu,
                Panel::Memory,
                Panel::Linux,
                Panel::Disk,
                Panel::Alerts,
            ][..],
            SpecialistView::Network => &[
                Panel::System,
                Panel::Cpu,
                Panel::Linux,
                Panel::Network,
                Panel::Alerts,
            ][..],
            SpecialistView::Jvm => &[Panel::Cpu, Panel::Memory, Panel::Alerts, Panel::Process][..],
            SpecialistView::DiskPressure => &[
                Panel::Memory,
                Panel::Linux,
                Panel::Disk,
                Panel::Alerts,
                Panel::Process,
            ][..],
            SpecialistView::DiskInventory => {
                &[Panel::System, Panel::Linux, Panel::Disk, Panel::Alerts][..]
            }
            SpecialistView::None => match self.operator_mode {
                OperatorMode::Overview => &[
                    Panel::System,
                    Panel::Cpu,
                    Panel::Memory,
                    Panel::Alerts,
                    Panel::Process,
                ][..],
                OperatorMode::Storage => &[
                    Panel::Memory,
                    Panel::Linux,
                    Panel::Disk,
                    Panel::Alerts,
                    Panel::Process,
                ][..],
                OperatorMode::Network => &[
                    Panel::System,
                    Panel::Cpu,
                    Panel::Linux,
                    Panel::Network,
                    Panel::Alerts,
                ][..],
                OperatorMode::Process => &[
                    Panel::System,
                    Panel::Cpu,
                    Panel::Memory,
                    Panel::Alerts,
                    Panel::Process,
                ][..],
                OperatorMode::Pressure => &[
                    Panel::Cpu,
                    Panel::Memory,
                    Panel::Linux,
                    Panel::Disk,
                    Panel::Alerts,
                ][..],
                OperatorMode::Full => &[
                    Panel::System,
                    Panel::Cpu,
                    Panel::Memory,
                    Panel::Linux,
                    Panel::Disk,
                    Panel::Network,
                    Panel::Alerts,
                    Panel::Process,
                ][..],
            },
        };

        let mut spans = Vec::new();
        for (index, panel) in panels.iter().copied().enumerate() {
            if index > 0 {
                spans.push(Span::raw("  "));
            }
            let (key, label, visible) = match panel {
                Panel::System => ("s", text(self.locale, "sys", "sys"), visibility.system),
                Panel::Cpu => ("c", text(self.locale, "cpu", "cpu"), visibility.cpu),
                Panel::Memory => ("m", text(self.locale, "mem", "mem"), visibility.memory),
                Panel::Linux => ("k", text(self.locale, "linux", "linux"), visibility.linux),
                Panel::Disk => ("d", text(self.locale, "disk", "disk"), visibility.disk),
                Panel::Network => ("n", text(self.locale, "net", "net"), visibility.network),
                Panel::Alerts => (
                    "a",
                    text(self.locale, "alertes", "alerts"),
                    visibility.alerts,
                ),
                Panel::Process => ("p", text(self.locale, "proc", "proc"), visibility.process),
            };
            spans.push(panel_toggle_span(key, label, visible, &self.theme));
        }

        Line::from(spans)
    }

    fn compact_status_line(
        &self,
        alerts: usize,
        visibility: &PanelVisibility,
        reference: &ReferenceUiState,
        logs: &LogUiState,
    ) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                format!(
                    "{}:{}/8  ",
                    text(self.locale, "visibles", "visible"),
                    visibility.visible_count()
                ),
                self.theme.highlight_style(),
            ),
            Span::styled(
                format!("{}:{}  ", text(self.locale, "alertes", "alerts"), alerts),
                if alerts > 0 {
                    self.theme.alert_style()
                } else {
                    self.theme.highlight_style()
                },
            ),
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
            Span::raw("  "),
            Span::styled(
                format!(
                    "{}:{}",
                    text(self.locale, "logs", "logs"),
                    logs.targets.len()
                ),
                if logs.targets.is_empty() {
                    self.theme.muted_style()
                } else {
                    self.theme.highlight_style()
                },
            ),
        ])
    }

    pub fn refresh_logs(
        &self,
        logs: &mut LogUiState,
        recent_file_secs: u64,
        max_files: usize,
        max_lines_per_file: usize,
    ) {
        let refresh = log_sources::refresh_tailed_paths(
            &logs.targets,
            &mut logs.file_states,
            recent_file_secs,
            max_files,
            max_lines_per_file,
        );
        logs.entries = refresh.entries;
        logs.active_files = refresh.active_files;
        logs.rotated_files = refresh.rotated_files;
        logs.last_refresh_ts = chrono::Utc::now().timestamp();
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

fn hotkey_span(key: &str, style: ratatui::style::Style) -> Span<'static> {
    Span::styled(format!(" {key}"), style)
}

fn merge_line(left: Line<'static>, right: Line<'static>) -> Line<'static> {
    Line::from(
        left.spans
            .into_iter()
            .chain(vec![Span::raw("  ")])
            .chain(right.spans)
            .collect::<Vec<_>>(),
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
            Self::Storage => text(locale, "io", "io"),
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

impl SpecialistView {
    fn reference_bias(self, panel: &str, category: &str, id: &str, index: usize) -> usize {
        let preferred = match self {
            SpecialistView::None => false,
            SpecialistView::Pressure => {
                matches!(panel, "memory" | "linux" | "disk" | "alerts")
                    || matches!(category, "memory" | "linux" | "disk")
                    || id.contains("pressure")
                    || id.contains("psi")
            }
            SpecialistView::Network => {
                matches!(panel, "network" | "alerts")
                    || matches!(category, "network")
                    || id.contains("tcp")
                    || id.contains("udp")
            }
            SpecialistView::Jvm => {
                matches!(panel, "process" | "cpu" | "memory")
                    || matches!(category, "process")
                    || id.contains("jvm")
                    || id.contains("thread")
            }
            SpecialistView::DiskPressure => {
                matches!(panel, "disk" | "process" | "linux" | "alerts")
                    || matches!(category, "disk" | "process")
                    || id.contains("await")
                    || id.contains("queue")
                    || id.contains("latency")
            }
            SpecialistView::DiskInventory => {
                matches!(panel, "disk" | "system")
                    || matches!(category, "disk")
                    || id.contains("filesystem")
                    || id.contains("inventory")
                    || id.contains("parent")
                    || id.contains("reference")
                    || id.contains("stack")
            }
        };

        if preferred {
            20_000usize.saturating_sub(index)
        } else {
            0
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
