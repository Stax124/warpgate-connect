use std::sync::{Arc, Mutex};

use crate::{
    app_data::ConnectionType,
    config::DEFAULT_WARPGATE_PORT,
    event::{AppEvent, Event, EventHandler},
    theme,
    utils::{MatchedTarget, filter_targets, rank_names},
    warpgate::structs::WarpgateTargetGroup,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    style::{Modifier, Style},
    widgets::TableState,
};
use ratatui_textarea::{Input, TextArea};
use strum::{EnumIter, IntoEnumIterator};
use tui_logger::TuiWidgetState;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AppScreen {
    Main,
    WarpgateSettings,
    Logs,
}

/// Layered over the current screen rather than replacing it, so the user keeps sight of what they
/// picked and `Esc` always has somewhere to go back to.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Modal {
    Connect,
    GroupPicker,
    Help,
}

/// Declaration order is the order the inputs are drawn in and the order `Tab` walks them.
#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter)]
pub enum WarpgateSettingsScreenInput {
    Url,
    Username,
    Token,
    Port,
}

impl WarpgateSettingsScreenInput {
    pub fn label(self) -> &'static str {
        match self {
            Self::Url => "URL",
            Self::Username => "Username",
            Self::Token => "Token",
            Self::Port => "Port",
        }
    }
}

#[derive(Debug)]
pub struct AppInputs<'a> {
    pub search_input: TextArea<'a>,
    pub group_picker_input: TextArea<'a>,
    pub warpgate_url_input: TextArea<'a>,
    pub warpgate_token_input: TextArea<'a>,
    pub warpgate_username_input: TextArea<'a>,
    pub warpgate_port_input: TextArea<'a>,
}

fn build_input(placeholder: &str, value: &str) -> TextArea<'static> {
    let mut text_area = TextArea::new(vec![value.to_string()]);
    text_area.set_placeholder_text(placeholder);
    text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::BOLD));
    text_area.set_placeholder_style(Style::default().add_modifier(Modifier::DIM));
    text_area
}

impl AppInputs<'_> {
    pub fn new(
        warpgate_url: &str,
        warpgate_token: &str,
        warpgate_username: &str,
        warpgate_port: &str,
    ) -> Self {
        let mut search_input = build_input("Type to search...", "");
        search_input
            .set_cursor_line_style(Style::default().fg(theme::KEY).add_modifier(Modifier::BOLD));

        let mut warpgate_token_input = build_input("Warpgate Token...", warpgate_token);
        warpgate_token_input.set_mask_char('●');

        let mut group_picker_input = build_input("filter groups...", "");
        group_picker_input
            .set_cursor_line_style(Style::default().fg(theme::KEY).add_modifier(Modifier::BOLD));

        Self {
            search_input,
            group_picker_input,
            warpgate_url_input: build_input("Warpgate URL...", warpgate_url),
            warpgate_token_input,
            warpgate_username_input: build_input("Warpgate Username...", warpgate_username),
            warpgate_port_input: build_input(&DEFAULT_WARPGATE_PORT.to_string(), warpgate_port),
        }
    }
}

fn host_from_config(warpgate_url: &str) -> String {
    crate::utils::get_domain_from_warpgate_url(warpgate_url).unwrap_or_default()
}

/// Keeps the highlighted row inside its list, which `TableState` alone does not do — `select_next`
/// and `select_last` can both leave the selection past the end.
fn clamp_selection(state: &mut TableState, len: usize) {
    state.select(match (len, state.selected()) {
        (0, _) => None,
        (len, Some(index)) => Some(index.min(len - 1)),
        (_, None) => Some(0),
    });
}

#[derive(Debug, Clone)]
pub struct GroupRow {
    /// `None` is the `all` row.
    pub group: Option<WarpgateTargetGroup>,
    pub count: usize,
}

pub struct App<'a> {
    pub running: bool,
    pub screen: AppScreen,
    pub modal: Option<Modal>,
    pub table_targets_selection_state: TableState,
    pub table_connection_selection_state: TableState,
    pub events: EventHandler,
    pub data: crate::app_data::Data,
    pub config: Arc<Mutex<crate::config::AppConfig>>,
    pub group_filter: Option<WarpgateTargetGroup>,
    pub group_picker_rows: Vec<GroupRow>,
    /// Indices into `group_picker_rows`, in the order the picker's own query ranked them.
    pub group_picker_matches: Vec<usize>,
    pub table_group_picker_state: TableState,
    pub ui_inputs: AppInputs<'a>,
    pub warpgate_selected_input: WarpgateSettingsScreenInput,
    pub filtered_targets: Vec<MatchedTarget>,
    pub total_ssh_targets: usize,
    /// Cached so the header does not lock the config and re-parse the URL every frame.
    pub warpgate_host: String,
    pub skip_update: bool,
    pub logger_state: TuiWidgetState,
}

impl<'a> App<'a> {
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
                    .unwrap_or(DEFAULT_WARPGATE_PORT)
                    .to_string(),
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

        let mut table_connection_selection_state = TableState::default();
        table_connection_selection_state.select_first();

        Self {
            data,
            config,
            running: true,
            screen,
            modal: None,
            table_targets_selection_state: TableState::default(),
            table_connection_selection_state,
            events: EventHandler::new(),
            group_filter: None,
            group_picker_rows: Vec::new(),
            group_picker_matches: Vec::new(),
            table_group_picker_state: TableState::default(),
            ui_inputs: AppInputs::new(
                warpgate_url.as_str(),
                warpgate_token.as_str(),
                warpgate_username.as_str(),
                warpgate_port.as_str(),
            ),
            warpgate_selected_input: WarpgateSettingsScreenInput::Url,
            filtered_targets: Vec::new(),
            total_ssh_targets: 0,
            warpgate_host: host_from_config(&warpgate_url),
            skip_update,
            logger_state: TuiWidgetState::new(),
        }
    }

    fn queue_startup_events(&mut self) {
        self.warpgate_update_input_focus();
        self.warpgate_update_input_validation();

        // Unconditional, including when starting on the settings screen: the fetch is what clears
        // `loading_targets` and records why an unconfigured install has no targets.
        self.events.send(AppEvent::RefreshTargets);

        if !self.skip_update {
            self.events.send(AppEvent::CheckForUpdate);
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        tracing::info!(screen = ?self.screen, "Application main loop started");

        self.queue_startup_events();

        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.events.next().await? {
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
                            .and_then(|index| self.filtered_targets.get(index))
                            .map(|matched| matched.target.clone());

                        match selected_target {
                            Some(target) => {
                                tracing::info!(target = %target.name, "Target selected");
                                self.data.selected_target.lock().unwrap().replace(target);
                                self.table_connection_selection_state.select_first();
                                self.modal = Some(Modal::Connect);
                            }
                            None => tracing::warn!(
                                "Target selection triggered but no target is highlighted"
                            ),
                        }
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
                            if let Some(version) = crate::update::check_for_newer_version() {
                                let _ = sender.send(Event::App(AppEvent::UpdateAvailable(version)));
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
        // F1 rather than `?`, so printable characters stay available to the focused input.
        if key_event.code == KeyCode::F(1) {
            self.modal = match self.modal {
                Some(Modal::Help) => None,
                _ => Some(Modal::Help),
            };
            return Ok(());
        }

        // Shortcuts live behind Control so that every printable character stays available to the
        // focused text input.
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c' | 'C' | 'q' | 'Q') => {
                    self.events.send(AppEvent::Quit);
                    return Ok(());
                }
                KeyCode::Char('r' | 'R') => {
                    self.events.send(AppEvent::RefreshTargets);
                    return Ok(());
                }
                KeyCode::Char('n' | 'N') if self.modal.is_none() => {
                    self.screen = match self.screen {
                        AppScreen::Main => AppScreen::WarpgateSettings,
                        AppScreen::WarpgateSettings => AppScreen::Logs,
                        AppScreen::Logs => AppScreen::Main,
                    };
                    tracing::debug!(screen = ?self.screen, "Switched screen");
                    return Ok(());
                }
                KeyCode::Char('u' | 'U') => {
                    if self.data.update_available.lock().unwrap().is_some() {
                        self.events.send(AppEvent::TriggerUpdate);
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        if let Some(modal) = self.modal {
            return self.handle_key_modal(modal, key_event);
        }

        match self.screen {
            AppScreen::Main => self.handle_key_main(key_event)?,
            AppScreen::WarpgateSettings => self.handle_key_warpgate_settings(key_event)?,
            AppScreen::Logs => {}
        }
        Ok(())
    }

    fn handle_key_modal(&mut self, modal: Modal, key_event: KeyEvent) -> color_eyre::Result<()> {
        if key_event.code == KeyCode::Esc {
            self.modal = None;
            return Ok(());
        }

        match modal {
            Modal::Connect => self.handle_key_connection_type(key_event)?,
            Modal::GroupPicker => self.handle_key_group_picker(key_event),
            Modal::Help => {}
        }
        Ok(())
    }

    pub fn handle_key_main(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('g' | 'G') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_group_picker()
            }
            KeyCode::Esc => {
                self.ui_inputs.search_input.select_all();
                self.ui_inputs.search_input.cut();
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

    fn open_group_picker(&mut self) {
        let mut counted_groups: Vec<(WarpgateTargetGroup, usize)> = Vec::new();
        let total_ssh_targets;
        {
            let warpgate_targets_guard = self.data.warpgate_targets.lock().unwrap();
            let targets = warpgate_targets_guard.as_deref().unwrap_or(&[]);
            total_ssh_targets = targets.iter().filter(|t| t.is_ssh()).count();

            for group in targets
                .iter()
                .filter(|t| t.is_ssh())
                .filter_map(|t| t.group.as_ref())
            {
                match counted_groups
                    .iter_mut()
                    .find(|(g, _)| g.name == group.name)
                {
                    Some((_, count)) => *count += 1,
                    None => counted_groups.push((group.clone(), 1)),
                }
            }
        }
        counted_groups.sort_by(|a, b| a.0.name.cmp(&b.0.name));

        self.group_picker_rows = std::iter::once(GroupRow {
            group: None,
            count: total_ssh_targets,
        })
        .chain(counted_groups.into_iter().map(|(group, count)| GroupRow {
            group: Some(group),
            count,
        }))
        .collect();

        self.ui_inputs.group_picker_input.select_all();
        self.ui_inputs.group_picker_input.cut();
        self.recalculate_group_picker_matches();

        let current_row = self
            .group_picker_rows
            .iter()
            .position(
                |row| match (row.group.as_ref(), self.group_filter.as_ref()) {
                    (None, None) => true,
                    (Some(row_group), Some(filter)) => row_group.name == filter.name,
                    _ => false,
                },
            )
            .unwrap_or(0);
        self.table_group_picker_state.select(
            self.group_picker_matches
                .iter()
                .position(|index| *index == current_row)
                .or(Some(0)),
        );

        self.modal = Some(Modal::GroupPicker);
    }

    fn recalculate_group_picker_matches(&mut self) {
        let query = self
            .ui_inputs
            .group_picker_input
            .lines()
            .first()
            .cloned()
            .unwrap_or_default();

        self.group_picker_matches = rank_names(
            self.group_picker_rows.iter().map(|row| {
                row.group
                    .as_ref()
                    .map_or("all", |group| group.name.as_str())
            }),
            &query,
        );

        clamp_selection(
            &mut self.table_group_picker_state,
            self.group_picker_matches.len(),
        );
    }

    fn handle_key_group_picker(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                self.apply_group_picker_selection();
                self.modal = None;
            }
            _ => {
                self.handle_table_input(key_event);
                self.handle_input(key_event);
            }
        }
    }

    fn apply_group_picker_selection(&mut self) {
        let selected_row = self
            .table_group_picker_state
            .selected()
            .and_then(|position| self.group_picker_matches.get(position))
            .and_then(|index| self.group_picker_rows.get(*index));

        let Some(row) = selected_row else {
            tracing::warn!("Group filter confirmed but no group is highlighted");
            return;
        };

        self.group_filter = row.group.clone();
        self.recalculate_filtered_targets();
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

                self.warpgate_host = host_from_config(warpgate_url.as_deref().unwrap_or_default());

                {
                    let mut config = self.config.lock().unwrap();
                    config.warpgate_api_url = warpgate_url;
                    config.warpgate_token = warpgate_token;
                    config.warpgate_username = warpgate_username;
                    config.warpgate_port = warpgate_port;
                    config.save()?;
                }

                tracing::info!("Warpgate settings saved");
                self.screen = AppScreen::Main;
                self.events.send(AppEvent::RefreshTargets);
            }

            KeyCode::Esc => self.screen = AppScreen::Main,
            KeyCode::Down | KeyCode::Tab => self.cycle_input(true),
            KeyCode::Up | KeyCode::BackTab => self.cycle_input(false),
            _ => self.handle_input(key_event),
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        tracing::info!("Application quitting");
        self.running = false;
    }

    pub fn cycle_input(&mut self, forward: bool) {
        let count = WarpgateSettingsScreenInput::iter().count();
        let current = WarpgateSettingsScreenInput::iter()
            .position(|input| input == self.warpgate_selected_input)
            .unwrap_or(0);
        let offset = if forward { 1 } else { count - 1 };

        self.warpgate_selected_input = WarpgateSettingsScreenInput::iter()
            .nth((current + offset) % count)
            .unwrap_or(WarpgateSettingsScreenInput::Url);
        self.warpgate_update_input_focus();
    }

    pub fn get_warpgate_input_by_enum(
        &mut self,
        input: WarpgateSettingsScreenInput,
    ) -> &mut TextArea<'a> {
        match input {
            WarpgateSettingsScreenInput::Url => &mut self.ui_inputs.warpgate_url_input,
            WarpgateSettingsScreenInput::Token => &mut self.ui_inputs.warpgate_token_input,
            WarpgateSettingsScreenInput::Username => &mut self.ui_inputs.warpgate_username_input,
            WarpgateSettingsScreenInput::Port => &mut self.ui_inputs.warpgate_port_input,
        }
    }

    /// Focus shows as the visible cursor and an accented label; the cursor *line* style is left
    /// to `warpgate_update_input_validation`, which would otherwise fight over it.
    pub fn warpgate_update_input_focus(&mut self) {
        let selected = self.warpgate_selected_input;
        for input in WarpgateSettingsScreenInput::iter() {
            let is_selected = input == selected;

            self.get_warpgate_input_by_enum(input)
                .set_cursor_style(match is_selected {
                    true => Style::default().add_modifier(Modifier::REVERSED),
                    false => Style::default(),
                });
        }
    }

    /// Colours the URL and token inputs by whether their current contents look usable.
    pub fn warpgate_update_input_validation(&mut self) {
        let url_is_valid = Self::get_string_from_textarea(&self.ui_inputs.warpgate_url_input)
            .is_some_and(|url| url.starts_with("http://") || url.starts_with("https://"));
        let token_is_valid =
            Self::get_string_from_textarea(&self.ui_inputs.warpgate_token_input).is_some();

        for (input, is_valid) in [
            (WarpgateSettingsScreenInput::Url, url_is_valid),
            (WarpgateSettingsScreenInput::Token, token_is_valid),
        ] {
            self.get_warpgate_input_by_enum(input)
                .set_cursor_line_style(Style::default().fg(if is_valid {
                    theme::TEXT
                } else {
                    theme::ERROR
                }));
        }
    }

    /// The one text input a keystroke should land in.
    fn focused_input(&mut self) -> Option<&mut TextArea<'a>> {
        if self.modal == Some(Modal::GroupPicker) {
            return Some(&mut self.ui_inputs.group_picker_input);
        }

        let selected = self.warpgate_selected_input;
        match self.screen {
            AppScreen::Main => Some(&mut self.ui_inputs.search_input),
            AppScreen::WarpgateSettings => Some(self.get_warpgate_input_by_enum(selected)),
            AppScreen::Logs => None,
        }
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        use ratatui_textarea::Key;

        let control = key_event.modifiers.contains(KeyModifiers::CONTROL);
        let Some(target_input) = self.focused_input() else {
            return;
        };

        let key = match key_event.code {
            KeyCode::Char('a') if control => {
                target_input.select_all();
                return;
            }
            // Unhandled Control chords must not be typed into the input as plain characters.
            KeyCode::Char(c) if !control => Key::Char(c),
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Backspace => Key::Backspace,
            _ => return,
        };

        target_input.input(Input {
            ctrl: control && matches!(key, Key::Left | Key::Right),
            alt: key_event.modifiers.contains(KeyModifiers::ALT),
            shift: key_event.modifiers.contains(KeyModifiers::SHIFT),
            key,
        });

        // Only the keys that edit the text can change what the query matches; moving the cursor
        // must not pay for a re-rank.
        if !matches!(key, Key::Char(_) | Key::Backspace) {
            return;
        }

        if self.modal == Some(Modal::GroupPicker) {
            self.recalculate_group_picker_matches();
            return;
        }

        match self.screen {
            AppScreen::Main => self.recalculate_filtered_targets(),
            AppScreen::WarpgateSettings => self.warpgate_update_input_validation(),
            AppScreen::Logs => {}
        }
    }

    pub fn recalculate_filtered_targets(&mut self) {
        let query = self
            .ui_inputs
            .search_input
            .lines()
            .first()
            .cloned()
            .unwrap_or_default();

        let (total_ssh_targets, filtered_targets) = {
            let warpgate_targets_guard = self.data.warpgate_targets.lock().unwrap();
            let targets = warpgate_targets_guard.as_deref().unwrap_or(&[]);
            (
                targets.iter().filter(|t| t.is_ssh()).count(),
                filter_targets(targets, self.group_filter.as_ref(), &query),
            )
        };
        self.total_ssh_targets = total_ssh_targets;
        self.filtered_targets = filtered_targets;

        clamp_selection(
            &mut self.table_targets_selection_state,
            self.filtered_targets.len(),
        );

        tracing::debug!(
            count = self.filtered_targets.len(),
            query = %query,
            group = ?self.group_filter.as_ref().map(|g| &g.name),
            "Recalculated filtered targets"
        );
    }

    pub fn handle_table_input(&mut self, key_event: KeyEvent) {
        // `None` where the row count is fixed by the table itself and never narrows.
        let (current_table_state, len) = match (self.modal, self.screen) {
            (Some(Modal::Connect), _) => (&mut self.table_connection_selection_state, None),
            (Some(Modal::GroupPicker), _) => (
                &mut self.table_group_picker_state,
                Some(self.group_picker_matches.len()),
            ),
            (None, AppScreen::Main) => (
                &mut self.table_targets_selection_state,
                Some(self.filtered_targets.len()),
            ),
            _ => return,
        };

        match key_event.code {
            KeyCode::Down => current_table_state.select_next(),
            KeyCode::Up => current_table_state.select_previous(),
            KeyCode::Home => current_table_state.select_first(),
            KeyCode::End => current_table_state.select_last(),
            _ => return,
        }

        if let Some(len) = len {
            clamp_selection(current_table_state, len);
        }
    }

    pub fn handle_key_connection_type(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Enter => {
                let selected_connection_type = self
                    .table_connection_selection_state
                    .selected()
                    .and_then(|index| ConnectionType::iter().nth(index));

                match selected_connection_type {
                    Some(connection_type) => self
                        .events
                        .send(AppEvent::ConnectionTypeSelected(connection_type)),
                    None => tracing::warn!(
                        "Connection type selection triggered but no connection type is highlighted"
                    ),
                }
            }
            _ => self.handle_table_input(key_event),
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "app_test.rs"]
mod tests;
