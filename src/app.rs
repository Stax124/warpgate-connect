use std::sync::{Arc, Mutex};

use crate::{
    event::{AppEvent, Event, EventHandler},
    warpgate::structs::{WarpgateFilterableTarget, WarpgateTarget, WarpgateTargetGroup},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nucleo_matcher::pattern::{CaseMatching, Normalization};
use ratatui::{
    DefaultTerminal,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Padding, TableState},
};
use ratatui_textarea::{Input, TextArea};
use strum::{EnumIter, IntoEnumIterator};
use tui_logger::TuiWidgetState;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AppScreen {
    Main,
    WarpgateSettings,
    Logs,
    ConnectionSelection,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter)]
pub enum WarpgateSettingsScreenInput {
    Url,
    Token,
    Username,
    Port,
}

#[derive(Debug)]
pub struct AppInputs<'a> {
    pub search_input: TextArea<'a>,
    pub warpgate_url_input: TextArea<'a>,
    pub warpgate_token_input: TextArea<'a>,
    pub warpgate_username_input: TextArea<'a>,
    pub warpgate_port_input: TextArea<'a>,
}

fn get_textarea_block(title: &str) -> ratatui::widgets::Block<'_> {
    ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Left)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::new().add_modifier(Modifier::BOLD))
        .padding(Padding::horizontal(1))
}

impl<'a> AppInputs<'a> {
    pub fn new(
        warpgate_url: &str,
        warpgate_token: &str,
        warpgate_username: &str,
        warpgate_port: &str,
    ) -> Self {
        Self {
            search_input: {
                let mut text_area = TextArea::default();
                text_area.set_block(get_textarea_block(" Search "));
                text_area.set_placeholder_text("Type to search...");
                text_area.set_cursor_line_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
                text_area.set_placeholder_style(Style::default().add_modifier(Modifier::DIM));
                text_area
            },
            warpgate_url_input: {
                let mut text_area = TextArea::new(vec![warpgate_url.to_string()]);
                text_area.set_block(get_textarea_block(" Warpgate URL "));
                text_area.set_placeholder_text("Warpgate URL...");
                text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::BOLD));
                text_area.set_placeholder_style(Style::default().add_modifier(Modifier::DIM));

                text_area
            },
            warpgate_token_input: {
                let mut text_area = TextArea::new(vec![warpgate_token.to_string()]);
                text_area.set_block(get_textarea_block(" Warpgate Token "));
                text_area.set_placeholder_text("Warpgate Token...");
                text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::BOLD));
                text_area.set_placeholder_style(Style::default().add_modifier(Modifier::DIM));

                text_area.set_mask_char('●');
                text_area
            },
            warpgate_username_input: {
                let mut text_area = TextArea::new(vec![warpgate_username.to_string()]);
                text_area.set_block(get_textarea_block(" Warpgate Username "));
                text_area.set_placeholder_text("Warpgate Username...");
                text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::BOLD));
                text_area.set_placeholder_style(Style::default().add_modifier(Modifier::DIM));
                text_area
            },
            warpgate_port_input: {
                let mut text_area = TextArea::new(vec![warpgate_port.to_string()]);
                text_area.set_block(get_textarea_block(" Warpgate Port "));
                text_area.set_placeholder_text("2222");
                text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::BOLD));
                text_area.set_placeholder_style(Style::default().add_modifier(Modifier::DIM));
                text_area
            },
        }
    }
}

pub struct App<'a> {
    pub running: bool,
    pub screen: AppScreen,
    pub table_targets_selection_state: TableState,
    pub table_connection_selection_state: TableState,
    pub events: EventHandler,
    pub data: crate::app_data::Data,
    pub config: Arc<Mutex<crate::config::AppConfig>>,
    pub group_filter: Option<WarpgateTargetGroup>,
    pub ui_inputs: AppInputs<'a>,
    pub warpgate_selected_input: WarpgateSettingsScreenInput,
    pub filtered_targets: Vec<WarpgateTarget>,
    pub skip_update: bool,
    pub logger_state: TuiWidgetState,
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new(
        data: crate::app_data::Data,
        config: Arc<Mutex<crate::config::AppConfig>>,
        skip_update: bool,
    ) -> Self {
        let (warpgate_url, warpgate_token, warpgate_username, warpgate_port) = {
            let cfg = config.lock().unwrap();
            (
                cfg.warpgate_api_url.clone().unwrap_or_default(),
                cfg.warpgate_token.clone().unwrap_or_default(),
                cfg.warpgate_username.clone().unwrap_or_default(),
                cfg.warpgate_port
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "2222".to_string()),
            )
        };

        let screen = {
            if warpgate_url.is_empty() || warpgate_token.is_empty() || warpgate_username.is_empty()
            {
                tracing::warn!("Missing warpgate configuration, opening settings screen");
                AppScreen::WarpgateSettings
            } else {
                tracing::debug!("Configuration loaded, starting on main screen");
                AppScreen::Main
            }
        };

        // Pre-select the first element in the connection selection screen
        let mut table_connection_selection_state = TableState::default();
        table_connection_selection_state.select_first();

        Self {
            data,
            config,
            running: true,
            screen,
            table_targets_selection_state: TableState::default(),
            table_connection_selection_state,
            events: EventHandler::new(),
            group_filter: None,
            ui_inputs: AppInputs::new(
                warpgate_url.as_str(),
                warpgate_token.as_str(),
                warpgate_username.as_str(),
                warpgate_port.as_str(),
            ),
            warpgate_selected_input: WarpgateSettingsScreenInput::Url,
            filtered_targets: Vec::new(),
            skip_update,
            logger_state: TuiWidgetState::new(),
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        tracing::info!(screen = ?self.screen, "Application main loop started");

        // Update the input borders to reflect the initially selected input
        self.warpgate_update_input_border();

        // Trigger initial fetch if we're on the main screen (config is valid)
        if self.screen == AppScreen::Main {
            self.events.send(AppEvent::RefreshTargets);
        }

        // Check for updates in the background
        if !self.skip_update {
            self.events.send(AppEvent::CheckForUpdate);
        }

        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.events.next().await? {
                Event::Render => {} // handled by draw above
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_global(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::TargetSelected => {
                        let selected_target = self
                            .table_targets_selection_state
                            .selected()
                            .and_then(|index| self.filtered_targets.get(index));

                        if let Some(target) = selected_target {
                            tracing::info!(target = %target.name, "Target selected");
                            self.data
                                .selected_target
                                .lock()
                                .unwrap()
                                .replace(target.clone());
                        } else {
                            tracing::warn!(
                                "Target selection triggered but no target is highlighted"
                            );
                        }

                        self.screen = AppScreen::ConnectionSelection;
                    }
                    AppEvent::ConnectionTypeSelected(connection_type) => {
                        tracing::info!(connection_type = ?connection_type, "Connection type selected");
                        self.data
                            .selected_connection_type
                            .lock()
                            .unwrap()
                            .replace(connection_type);

                        self.quit();
                    }
                    AppEvent::RefreshTargets => {
                        tracing::info!("Refreshing warpgate targets");
                        let data = self.data.clone();
                        let config = self.config.clone();
                        let sender = self.events.sender.clone();

                        tokio::spawn(async move {
                            crate::warpgate::fetch::fetch_warpgate_data(data, config).await;
                            let _ = sender.send(Event::App(AppEvent::RecalculateTargets));
                        });
                    }
                    AppEvent::RecalculateTargets => {
                        self.recalculate_filtered_targets();
                    }
                    AppEvent::CheckForUpdate => {
                        tracing::info!("Checking for application updates");
                        let sender = self.events.sender.clone();

                        tokio::task::spawn_blocking(move || {
                            let mut updater_base =
                                self_update::backends::github::Update::configure();

                            updater_base
                                .repo_owner("stax124")
                                .repo_name("warpgate-connect")
                                .bin_name("warpgate-connect")
                                .current_version(env!("CARGO_PKG_VERSION"));

                            let updater = updater_base.build();
                            if let Ok(updater) = updater
                                && let Ok(release) = updater.get_latest_release()
                            {
                                let current = env!("CARGO_PKG_VERSION");
                                if self_update::version::bump_is_greater(current, &release.version)
                                    .unwrap_or(false)
                                {
                                    let _ = sender.send(Event::App(AppEvent::UpdateAvailable(
                                        release.version,
                                    )));
                                }
                            }
                        });
                    }
                    AppEvent::UpdateAvailable(version) => {
                        tracing::info!(version = %version, "Update available");
                        *self.data.update_available.lock().unwrap() = Some(version);
                    }
                    AppEvent::TriggerUpdate => {
                        tracing::info!("User triggered update");
                        *self.data.trigger_update.lock().unwrap() = true;
                        self.quit();
                    }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_global(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char('Q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('R') => self.events.send(AppEvent::RefreshTargets),
            KeyCode::Char('U') => {
                if self.data.update_available.lock().unwrap().is_some() {
                    self.events.send(AppEvent::TriggerUpdate);
                }
            }
            KeyCode::Char('N') => {
                // Swap between screens
                self.screen = match self.screen {
                    AppScreen::Main => AppScreen::WarpgateSettings,
                    AppScreen::WarpgateSettings => AppScreen::Logs,
                    AppScreen::Logs => AppScreen::Main,
                    AppScreen::ConnectionSelection => AppScreen::ConnectionSelection,
                };
                tracing::debug!(screen = ?self.screen, "Switched screen");
            }
            _ => match self.screen {
                AppScreen::Main => self.handle_key_main(key_event)?,
                AppScreen::WarpgateSettings => self.handle_key_warpgate_settings(key_event)?,
                AppScreen::Logs => {}
                AppScreen::ConnectionSelection => self.handle_key_connection_type(key_event)?,
            },
        }
        Ok(())
    }

    pub fn handle_key_main(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('G') => {
                let mut available_groups: Vec<WarpgateTargetGroup> = self
                    .data
                    .warpgate_targets
                    .lock()
                    .unwrap()
                    .as_ref()
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|t| t.group.clone())
                    .collect();

                available_groups.sort_by(|a, b| a.name.cmp(&b.name));
                available_groups.dedup();

                self.group_filter = match &self.group_filter {
                    Some(current) => {
                        let next_idx = available_groups
                            .iter()
                            .position(|g| g == current)
                            .map(|idx| idx + 1)
                            .unwrap_or(0);
                        available_groups.get(next_idx).cloned()
                    }
                    None => available_groups.first().cloned(),
                };

                self.recalculate_filtered_targets();
            }
            KeyCode::Enter => self.events.send(AppEvent::TargetSelected),
            _ => {
                self.handle_table_input(key_event);
                self.handle_input(key_event);
            }
        }
        Ok(())
    }

    pub fn get_string_from_textarea(text_area: &TextArea) -> Option<String> {
        text_area.lines().first().cloned().and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }

    pub fn handle_key_warpgate_settings(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                let warpgate_url =
                    Self::get_string_from_textarea(&self.ui_inputs.warpgate_url_input);
                let warpgate_token =
                    Self::get_string_from_textarea(&self.ui_inputs.warpgate_token_input);
                let warpgate_username =
                    Self::get_string_from_textarea(&self.ui_inputs.warpgate_username_input);
                let warpgate_port =
                    Self::get_string_from_textarea(&self.ui_inputs.warpgate_port_input)
                        .and_then(|s| s.parse::<u16>().ok());

                let mut config = self.config.lock().unwrap();
                config.warpgate_api_url = warpgate_url;
                config.warpgate_token = warpgate_token;
                config.warpgate_username = warpgate_username;
                config.warpgate_port = warpgate_port;
                config.save()?;

                tracing::info!("Warpgate settings saved");
                self.screen = AppScreen::Main;
                self.events.send(AppEvent::RefreshTargets);
            }

            KeyCode::Down => self.warpgate_select_next_input(),
            KeyCode::Up => self.warpgate_select_previous_input(),
            KeyCode::BackTab => self.warpgate_select_previous_input(),
            KeyCode::Tab => self.warpgate_select_next_input(),
            _ => self.handle_input(key_event),
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        tracing::info!("Application quitting");
        self.running = false;
    }

    pub fn warpgate_select_next_input(&mut self) {
        self.warpgate_selected_input = match self.warpgate_selected_input {
            WarpgateSettingsScreenInput::Url => WarpgateSettingsScreenInput::Username,
            WarpgateSettingsScreenInput::Username => WarpgateSettingsScreenInput::Token,
            WarpgateSettingsScreenInput::Token => WarpgateSettingsScreenInput::Port,
            WarpgateSettingsScreenInput::Port => WarpgateSettingsScreenInput::Url,
        };

        self.warpgate_update_input_border();
    }

    pub fn warpgate_select_previous_input(&mut self) {
        self.warpgate_selected_input = match self.warpgate_selected_input {
            WarpgateSettingsScreenInput::Url => WarpgateSettingsScreenInput::Port,
            WarpgateSettingsScreenInput::Port => WarpgateSettingsScreenInput::Token,
            WarpgateSettingsScreenInput::Token => WarpgateSettingsScreenInput::Username,
            WarpgateSettingsScreenInput::Username => WarpgateSettingsScreenInput::Url,
        };

        self.warpgate_update_input_border();
    }

    pub fn get_warpgate_input_by_enum(
        &mut self,
        input: &WarpgateSettingsScreenInput,
    ) -> &mut TextArea<'a> {
        match input {
            WarpgateSettingsScreenInput::Url => &mut self.ui_inputs.warpgate_url_input,
            WarpgateSettingsScreenInput::Token => &mut self.ui_inputs.warpgate_token_input,
            WarpgateSettingsScreenInput::Username => &mut self.ui_inputs.warpgate_username_input,
            WarpgateSettingsScreenInput::Port => &mut self.ui_inputs.warpgate_port_input,
        }
    }

    pub fn warpgate_update_input_border(&mut self) {
        let selected = self.warpgate_selected_input;
        for input in WarpgateSettingsScreenInput::iter() {
            let is_selected = input == selected;
            let text_area = self.get_warpgate_input_by_enum(&input);

            text_area.set_block(
                get_textarea_block(match input {
                    WarpgateSettingsScreenInput::Url => " Warpgate URL ",
                    WarpgateSettingsScreenInput::Token => " Warpgate Token ",
                    WarpgateSettingsScreenInput::Username => " Warpgate Username ",
                    WarpgateSettingsScreenInput::Port => " Warpgate Port ",
                })
                .border_style(if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }),
            );

            text_area.set_cursor_style(match is_selected {
                true => Style::default().add_modifier(Modifier::REVERSED),
                false => Style::default(),
            });
        }
    }

    // Search query
    pub fn handle_input(&mut self, key_event: KeyEvent) {
        let target_input = match self.screen {
            AppScreen::Main => &mut self.ui_inputs.search_input,
            AppScreen::WarpgateSettings => match self.warpgate_selected_input {
                WarpgateSettingsScreenInput::Url => &mut self.ui_inputs.warpgate_url_input,
                WarpgateSettingsScreenInput::Token => &mut self.ui_inputs.warpgate_token_input,
                WarpgateSettingsScreenInput::Username => {
                    &mut self.ui_inputs.warpgate_username_input
                }
                WarpgateSettingsScreenInput::Port => &mut self.ui_inputs.warpgate_port_input,
            },
            AppScreen::Logs => return,
            AppScreen::ConnectionSelection => return,
        };

        match key_event.code {
            KeyCode::Char('a') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                target_input.select_all();
            }
            KeyCode::Char(c) => {
                target_input.input(Input {
                    key: ratatui_textarea::Key::Char(c),
                    ctrl: false,
                    alt: key_event.modifiers.contains(KeyModifiers::ALT),
                    shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
                });
            }
            KeyCode::Left => {
                target_input.input(Input {
                    key: ratatui_textarea::Key::Left,
                    ctrl: key_event.modifiers.contains(KeyModifiers::CONTROL),
                    alt: key_event.modifiers.contains(KeyModifiers::ALT),
                    shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
                });
            }
            KeyCode::Right => {
                target_input.input(Input {
                    key: ratatui_textarea::Key::Right,
                    ctrl: key_event.modifiers.contains(KeyModifiers::CONTROL),
                    alt: key_event.modifiers.contains(KeyModifiers::ALT),
                    shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
                });
            }
            KeyCode::Backspace => {
                target_input.input(Input {
                    key: ratatui_textarea::Key::Backspace,
                    ctrl: false,
                    alt: key_event.modifiers.contains(KeyModifiers::ALT),
                    shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
                });
            }
            _ => {}
        }

        // Trigger recalculation if we are on the main screen to update the filtered targets based on the search query
        if self.screen == AppScreen::Main {
            self.recalculate_filtered_targets();
        }
    }

    pub fn recalculate_filtered_targets(&mut self) {
        let mut matcher =
            nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT.match_paths());

        let warpgate_targets_guard = self.data.warpgate_targets.lock().unwrap();
        let query = self
            .ui_inputs
            .search_input
            .lines()
            .first()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let pre_filtered_targets = warpgate_targets_guard
            .as_ref()
            .ok()
            .into_iter()
            .flatten()
            // Only show SSH targets
            .filter(|t| t.kind == "Ssh")
            // Apply group filter if it exists
            .filter(|t| {
                if let Some(group_filter) = &self.group_filter {
                    t.group
                        .as_ref()
                        .is_some_and(|g| g.name == group_filter.name)
                } else {
                    true
                }
            })
            .cloned()
            .collect::<Vec<WarpgateTarget>>();

        let matches = nucleo_matcher::pattern::Pattern::parse(
            &query,
            CaseMatching::Ignore,
            Normalization::Smart,
        )
        .match_list(
            pre_filtered_targets
                .iter()
                .map(|t| WarpgateFilterableTarget::new(t.clone())),
            &mut matcher,
        );

        self.filtered_targets = matches.into_iter().map(|m| m.0.warpgate_target).collect();

        tracing::debug!(
            count = self.filtered_targets.len(),
            query = %query,
            group = ?self.group_filter.as_ref().map(|g| &g.name),
            "Recalculated filtered targets"
        );
    }

    pub fn handle_table_input(&mut self, key_event: KeyEvent) {
        let current_table_state = match self.screen {
            AppScreen::Main => &mut self.table_targets_selection_state,
            AppScreen::ConnectionSelection => &mut self.table_connection_selection_state,
            _ => return,
        };

        match key_event.code {
            KeyCode::Down => current_table_state.select_next(),
            KeyCode::Up => current_table_state.select_previous(),
            KeyCode::Home => current_table_state.select_first(),
            KeyCode::End => current_table_state.select_last(),
            _ => {}
        }
    }

    pub fn handle_key_connection_type(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                let selected_connection_type = match self
                    .table_connection_selection_state
                    .selected()
                {
                    Some(0) => crate::app_data::ConnectionType::Ssh,
                    Some(1) => crate::app_data::ConnectionType::Sftp,
                    _ => {
                        tracing::warn!(
                            "Connection type selection triggered but no connection type is highlighted"
                        );
                        return Ok(());
                    }
                };

                self.events
                    .send(AppEvent::ConnectionTypeSelected(selected_connection_type));
            }
            _ => self.handle_table_input(key_event),
        }
        Ok(())
    }
}
