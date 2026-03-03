pub mod dashboard;
pub mod i18n;
pub mod theme;
pub mod widgets;

use crate::collectors::Snapshot;
use crate::config::{Config, LogsConfig, TuiConfig};
use crate::engine::scheduler::TickEvent;
use crate::reference::Locale;
use crate::tui::widgets::analysis_widget::SpecialistView;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dashboard::{Dashboard, LogUiState, OperatorMode, Panel, ReferenceUiState};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::Path;
use std::time::Duration;
use tokio::sync::broadcast;

pub async fn run_tui(
    config: &TuiConfig,
    logs_config: &LogsConfig,
    config_path: &Path,
    mut rx: broadcast::Receiver<TickEvent>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut dashboard = Dashboard::new(&config.theme, Locale::parse(&config.locale));
    let mut reference = ReferenceUiState::default();
    let mut logs = LogUiState {
        targets: logs_config.paths.clone(),
        ..LogUiState::default()
    };
    let refresh = Duration::from_millis(config.refresh_rate_ms);
    let mut current_snapshot = Snapshot::default();

    loop {
        // Drainer la queue et garder uniquement le snapshot le plus récent
        loop {
            match rx.try_recv() {
                Ok(tick) => {
                    current_snapshot = tick.snapshot;
                    if logs.visible && !logs.input_active {
                        maybe_refresh_logs(&dashboard, logs_config, &mut logs);
                    }
                }
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => break,
                Err(broadcast::error::TryRecvError::Closed) => {
                    restore_terminal(&mut terminal)?;
                    return Ok(());
                }
            }
        }

        terminal.draw(|frame| dashboard.render(frame, &current_snapshot, &reference, &logs))?;

        if event::poll(refresh)? {
            if let Event::Key(key) = event::read()? {
                if logs.input_active {
                    match key.code {
                        KeyCode::Esc => {
                            logs.input_active = false;
                            logs.query.clear();
                            if logs.targets.is_empty() {
                                logs.visible = false;
                            }
                        }
                        KeyCode::Enter => {
                            let target = logs.query.trim();
                            if !target.is_empty() && !logs.targets.iter().any(|item| item == target)
                            {
                                logs.targets.push(target.to_string());
                                logs.targets.sort();
                                if let Err(error) = persist_log_targets(config_path, &logs.targets)
                                {
                                    logs.last_error = Some(error.to_string());
                                } else {
                                    logs.last_error = None;
                                }
                            }
                            logs.query.clear();
                            logs.input_active = false;
                            logs.visible = true;
                            dashboard.refresh_logs(
                                &mut logs,
                                logs_config.recent_file_secs,
                                logs_config.max_files,
                                logs_config.max_lines_per_file,
                            );
                        }
                        KeyCode::Backspace => {
                            logs.query.pop();
                        }
                        KeyCode::Char(ch) => {
                            logs.query.push(ch);
                        }
                        _ => {}
                    }
                    continue;
                }

                if reference.input_active {
                    match key.code {
                        KeyCode::Esc => {
                            reference.input_active = false;
                            if reference.query.is_empty() {
                                reference.visible = false;
                            }
                        }
                        KeyCode::Enter => {
                            reference.input_active = false;
                            reference.visible = true;
                        }
                        KeyCode::Backspace => {
                            reference.query.pop();
                            reference.visible = true;
                            reference.selected = 0;
                        }
                        KeyCode::Up => {
                            reference.selected = reference.selected.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            reference.selected = reference.selected.saturating_add(1);
                        }
                        KeyCode::Char(ch) => {
                            reference.query.push(ch);
                            reference.visible = true;
                            reference.selected = 0;
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('/') => {
                        logs.visible = false;
                        logs.input_active = false;
                        reference.visible = true;
                        reference.input_active = true;
                        reference.selected = 0;
                    }
                    KeyCode::Char('?') => {
                        logs.visible = false;
                        logs.input_active = false;
                        reference.visible = !reference.visible;
                        reference.input_active = false;
                        reference.selected = 0;
                    }
                    KeyCode::Char('l') => {
                        reference.visible = false;
                        reference.input_active = false;
                        logs.visible = true;
                        logs.input_active = logs.targets.is_empty();
                        if !logs.input_active {
                            dashboard.refresh_logs(
                                &mut logs,
                                logs_config.recent_file_secs,
                                logs_config.max_files,
                                logs_config.max_lines_per_file,
                            );
                        }
                        terminal.clear()?;
                    }
                    KeyCode::Char('L') => {
                        reference.visible = false;
                        reference.input_active = false;
                        logs.visible = true;
                        logs.input_active = true;
                        terminal.clear()?;
                    }
                    KeyCode::Esc => {
                        reference.input_active = false;
                        if logs.visible {
                            logs.visible = false;
                            logs.input_active = false;
                            logs.query.clear();
                        } else if reference.visible {
                            reference.visible = false;
                        } else {
                            reference.query.clear();
                            reference.selected = 0;
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        terminal.clear()?;
                    }
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        dashboard.cycle_locale();
                        terminal.clear()?;
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        dashboard.toggle_detail();
                        terminal.clear()?;
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        dashboard.cycle_theme();
                        terminal.clear()?;
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        dashboard.toggle_panel(Panel::Cpu);
                        terminal.clear()?;
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        dashboard.toggle_panel(Panel::System);
                        terminal.clear()?;
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        dashboard.toggle_panel(Panel::Memory);
                        terminal.clear()?;
                    }
                    KeyCode::Char('k') | KeyCode::Char('K') => {
                        dashboard.toggle_panel(Panel::Linux);
                        terminal.clear()?;
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        dashboard.toggle_panel(Panel::Disk);
                        terminal.clear()?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        dashboard.toggle_panel(Panel::Network);
                        terminal.clear()?;
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        dashboard.toggle_panel(Panel::Alerts);
                        terminal.clear()?;
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        dashboard.toggle_panel(Panel::Process);
                        terminal.clear()?;
                    }
                    KeyCode::Char('1') => {
                        dashboard.set_operator_mode(OperatorMode::Overview);
                        terminal.clear()?;
                    }
                    KeyCode::Char('2') => {
                        dashboard.set_operator_mode(OperatorMode::Storage);
                        terminal.clear()?;
                    }
                    KeyCode::Char('3') => {
                        dashboard.set_operator_mode(OperatorMode::Network);
                        terminal.clear()?;
                    }
                    KeyCode::Char('4') => {
                        dashboard.set_operator_mode(OperatorMode::Process);
                        terminal.clear()?;
                    }
                    KeyCode::Char('5') => {
                        dashboard.set_operator_mode(OperatorMode::Pressure);
                        terminal.clear()?;
                    }
                    KeyCode::Char('6') => {
                        dashboard.set_operator_mode(OperatorMode::Full);
                        terminal.clear()?;
                    }
                    KeyCode::Char('7') => {
                        dashboard.set_specialist_view(SpecialistView::Pressure);
                        terminal.clear()?;
                    }
                    KeyCode::Char('8') => {
                        dashboard.set_specialist_view(SpecialistView::Network);
                        terminal.clear()?;
                    }
                    KeyCode::Char('9') => {
                        dashboard.set_specialist_view(SpecialistView::Jvm);
                        terminal.clear()?;
                    }
                    KeyCode::Char('0') => {
                        dashboard.set_specialist_view(SpecialistView::DiskPressure);
                        terminal.clear()?;
                    }
                    KeyCode::Char('g') | KeyCode::Char('G') => {
                        dashboard.set_specialist_view(SpecialistView::DiskInventory);
                        terminal.clear()?;
                    }
                    KeyCode::Char('-') => {
                        dashboard.set_specialist_view(SpecialistView::None);
                        terminal.clear()?;
                    }
                    KeyCode::Up if reference.visible => {
                        reference.selected = reference.selected.saturating_sub(1);
                    }
                    KeyCode::Down if reference.visible => {
                        reference.selected = reference.selected.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn maybe_refresh_logs(dashboard: &Dashboard, config: &LogsConfig, logs: &mut LogUiState) {
    let now = chrono::Utc::now().timestamp();
    if now.saturating_sub(logs.last_refresh_ts) < 5 {
        return;
    }
    dashboard.refresh_logs(
        logs,
        config.recent_file_secs,
        config.max_files,
        config.max_lines_per_file,
    );
}

fn persist_log_targets(config_path: &Path, targets: &[String]) -> Result<()> {
    let mut config = Config::load_or_default(config_path);
    config.logs.paths = targets.to_vec();
    config.save(config_path)
}
