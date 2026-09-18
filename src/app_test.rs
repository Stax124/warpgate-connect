use super::*;
use crate::app_data::Data;
use crate::config::AppConfig;
use crate::warpgate::structs::WarpgateTarget;

fn target(name: &str) -> WarpgateTarget {
    WarpgateTarget {
        description: None,
        group: None,
        kind: "Ssh".to_string(),
        name: name.to_string(),
    }
}

fn grouped_target(name: &str, group: &str) -> WarpgateTarget {
    WarpgateTarget {
        group: Some(WarpgateTargetGroup {
            name: group.to_string(),
            id: format!("grp-{group}"),
            color: None,
        }),
        ..target(name)
    }
}

/// `AppConfig::default()` keeps this away from the real config file on disk.
fn test_app() -> App<'static> {
    App::new(
        Data::new(),
        Arc::new(Mutex::new(AppConfig::default())),
        true,
    )
}

fn set_search_query(app: &mut App, query: &str) {
    app.ui_inputs.search_input.select_all();
    app.ui_inputs.search_input.cut();
    app.ui_inputs.search_input.insert_str(query);
}

/// Guards the regression where narrowing the list left the highlight past its end, so `Enter`
/// resolved to no target yet still advanced to the connection screen.
#[tokio::test]
async fn narrowing_the_list_keeps_the_highlight_in_range() {
    let mut app = test_app();
    *app.data.warpgate_targets.lock().unwrap() =
        Ok(vec![target("alpha"), target("beta"), target("gamma")]);

    app.recalculate_filtered_targets();
    app.table_targets_selection_state.select(Some(2));

    set_search_query(&mut app, "alpha");
    app.recalculate_filtered_targets();

    let selected = app
        .table_targets_selection_state
        .selected()
        .expect("a highlight should survive the narrowing");
    assert!(
        app.filtered_targets.get(selected).is_some(),
        "highlight {selected} is outside a list of {}",
        app.filtered_targets.len()
    );
}

#[tokio::test]
async fn an_empty_result_leaves_nothing_highlighted() {
    let mut app = test_app();
    *app.data.warpgate_targets.lock().unwrap() = Ok(vec![target("alpha")]);

    app.recalculate_filtered_targets();
    assert_eq!(app.table_targets_selection_state.selected(), Some(0));

    set_search_query(&mut app, "no-such-target");
    app.recalculate_filtered_targets();

    assert!(app.filtered_targets.is_empty());
    assert_eq!(app.table_targets_selection_state.selected(), None);
}

#[tokio::test]
async fn the_first_target_is_highlighted_once_a_fetch_lands() {
    let mut app = test_app();
    assert_eq!(app.table_targets_selection_state.selected(), None);

    *app.data.warpgate_targets.lock().unwrap() = Ok(vec![target("alpha"), target("beta")]);
    app.recalculate_filtered_targets();

    assert_eq!(app.table_targets_selection_state.selected(), Some(0));
}

fn press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key_global(KeyEvent::new(code, modifiers))
        .expect("key handling should not fail");
}

/// Guards the regression where the global shortcuts sat on bare uppercase letters, so a token
/// containing `Q`, `R`, `N` or `U` could not be typed — `Shift+Q` quit the app mid-edit.
#[tokio::test]
async fn uppercase_letters_reach_the_focused_settings_input() {
    let mut app = test_app();
    app.screen = AppScreen::WarpgateSettings;
    app.warpgate_selected_input = WarpgateSettingsScreenInput::Token;

    for character in ['Q', 'R', 'N', 'U', 'G'] {
        press(&mut app, KeyCode::Char(character), KeyModifiers::SHIFT);
    }

    assert_eq!(
        app.ui_inputs.warpgate_token_input.lines().first().unwrap(),
        "QRNUG"
    );
    assert_eq!(app.screen, AppScreen::WarpgateSettings);
}

#[tokio::test]
async fn uppercase_letters_reach_the_search_box() {
    let mut app = test_app();
    app.screen = AppScreen::Main;

    press(&mut app, KeyCode::Char('G'), KeyModifiers::SHIFT);

    assert_eq!(app.ui_inputs.search_input.lines().first().unwrap(), "G");
    assert!(app.group_filter.is_none(), "Shift+G must not filter groups");
}

#[tokio::test]
async fn control_chords_are_shortcuts_rather_than_text() {
    let mut app = test_app();
    app.screen = AppScreen::WarpgateSettings;
    app.warpgate_selected_input = WarpgateSettingsScreenInput::Token;

    press(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL);

    assert_eq!(
        app.ui_inputs.warpgate_token_input.lines().first().unwrap(),
        "",
        "a Control chord must not be typed into the input"
    );

    let event = tokio::time::timeout(std::time::Duration::from_secs(5), app.events.next())
        .await
        .expect("Ctrl+Q should have queued an event")
        .expect("the event channel should be open");

    assert!(matches!(event, Event::App(AppEvent::Quit)));
}

#[tokio::test]
async fn control_n_cycles_screens() {
    let mut app = test_app();
    app.screen = AppScreen::Main;

    press(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(app.screen, AppScreen::WarpgateSettings);

    press(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(app.screen, AppScreen::Logs);

    press(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(app.screen, AppScreen::Main);
}

/// The settings inputs are walked in the order they are drawn, not in an order written out a
/// second time by hand.
#[tokio::test]
async fn tab_walks_the_settings_inputs_in_display_order() {
    let mut app = test_app();
    app.screen = AppScreen::WarpgateSettings;

    let mut visited = vec![app.warpgate_selected_input];
    for _ in 0..4 {
        press(&mut app, KeyCode::Tab, KeyModifiers::NONE);
        visited.push(app.warpgate_selected_input);
    }

    assert_eq!(
        visited,
        [
            WarpgateSettingsScreenInput::Url,
            WarpgateSettingsScreenInput::Username,
            WarpgateSettingsScreenInput::Token,
            WarpgateSettingsScreenInput::Port,
            WarpgateSettingsScreenInput::Url,
        ]
    );

    press(&mut app, KeyCode::BackTab, KeyModifiers::NONE);
    assert_eq!(
        app.warpgate_selected_input,
        WarpgateSettingsScreenInput::Port
    );
}

/// The modal is layered over the main screen, so a key that reaches it must not also land in the
/// search box underneath.
#[tokio::test]
async fn an_open_modal_takes_the_keys_from_the_screen_beneath_it() {
    let mut app = test_app();
    app.screen = AppScreen::Main;
    app.modal = Some(Modal::Connect);

    press(&mut app, KeyCode::Char('x'), KeyModifiers::NONE);
    assert_eq!(
        app.ui_inputs.search_input.lines().first().unwrap(),
        "",
        "a key handled by the modal must not be typed into the search box"
    );

    press(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.modal.is_none(), "Esc should close the modal");
}

#[tokio::test]
async fn esc_clears_the_search_on_the_main_screen() {
    let mut app = test_app();
    app.screen = AppScreen::Main;
    *app.data.warpgate_targets.lock().unwrap() = Ok(vec![target("alpha"), target("beta")]);

    set_search_query(&mut app, "alpha");
    app.recalculate_filtered_targets();
    assert_eq!(app.filtered_targets.len(), 1);

    press(&mut app, KeyCode::Esc, KeyModifiers::NONE);

    assert_eq!(app.ui_inputs.search_input.lines().first().unwrap(), "");
    assert_eq!(app.filtered_targets.len(), 2);
}

/// The modal decodes its highlighted row back into a `ConnectionType` by index, so this guards the
/// row order against a reordering of the enum.
#[tokio::test]
async fn the_connect_modal_decodes_the_highlighted_row() {
    let mut app = test_app();
    app.modal = Some(Modal::Connect);

    for (row, expected) in [(0, ConnectionType::Ssh), (1, ConnectionType::Sftp)] {
        app.table_connection_selection_state.select(Some(row));
        press(&mut app, KeyCode::Enter, KeyModifiers::NONE);

        let event = tokio::time::timeout(std::time::Duration::from_secs(5), app.events.next())
            .await
            .expect("Enter should have queued an event")
            .expect("the event channel should be open");

        assert!(
            matches!(event, Event::App(AppEvent::ConnectionTypeSelected(got)) if got == expected),
            "row {row} decoded to {event:?}, expected {expected}"
        );
    }
}

/// Guards the whole picker path: the counts it offers, the filter Enter applies, and the
/// highlight clamping that narrowing the list behind it forces.
#[tokio::test]
async fn the_group_picker_applies_a_filter_and_clamps_the_highlight() {
    let mut app = test_app();
    app.screen = AppScreen::Main;
    *app.data.warpgate_targets.lock().unwrap() = Ok(vec![
        grouped_target("prod-db", "production"),
        grouped_target("prod-web", "production"),
        grouped_target("staging-db", "staging"),
        target("orphan"),
    ]);
    app.recalculate_filtered_targets();
    app.table_targets_selection_state.select(Some(3));

    press(&mut app, KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert_eq!(app.modal, Some(Modal::GroupPicker));

    let offered: Vec<(Option<&str>, usize)> = app
        .group_picker_rows
        .iter()
        .map(|row| (row.group.as_ref().map(|g| g.name.as_str()), row.count))
        .collect();
    assert_eq!(
        offered,
        [(None, 4), (Some("production"), 2), (Some("staging"), 1),]
    );

    // Typing narrows the rows without leaving the highlight past the end of them.
    press(&mut app, KeyCode::Char('s'), KeyModifiers::NONE);
    press(&mut app, KeyCode::Char('t'), KeyModifiers::NONE);
    let selected = app
        .table_group_picker_state
        .selected()
        .expect("a row should stay highlighted");
    assert!(selected < app.group_picker_matches.len());

    press(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    assert!(app.modal.is_none());
    assert_eq!(
        app.group_filter.as_ref().map(|g| g.name.as_str()),
        Some("staging")
    );
    assert_eq!(app.filtered_targets.len(), 1);
    assert_eq!(app.table_targets_selection_state.selected(), Some(0));
}

#[tokio::test]
async fn the_group_picker_can_go_back_to_every_group() {
    let mut app = test_app();
    app.screen = AppScreen::Main;
    *app.data.warpgate_targets.lock().unwrap() = Ok(vec![
        grouped_target("prod-db", "production"),
        target("orphan"),
    ]);
    app.recalculate_filtered_targets();

    press(&mut app, KeyCode::Char('g'), KeyModifiers::CONTROL);
    press(&mut app, KeyCode::Down, KeyModifiers::NONE);
    press(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        app.group_filter.as_ref().map(|g| g.name.as_str()),
        Some("production")
    );

    press(&mut app, KeyCode::Char('g'), KeyModifiers::CONTROL);
    press(&mut app, KeyCode::Home, KeyModifiers::NONE);
    press(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    assert!(
        app.group_filter.is_none(),
        "the `all` row clears the filter"
    );
    assert_eq!(app.filtered_targets.len(), 2);
}

/// The footer advertises F1 on every screen, including the ones whose text fields would swallow a
/// printable key.
#[tokio::test]
async fn f1_toggles_the_help_overlay_from_anywhere() {
    let mut app = test_app();

    for screen in [
        AppScreen::Main,
        AppScreen::WarpgateSettings,
        AppScreen::Logs,
    ] {
        app.screen = screen;

        press(&mut app, KeyCode::F(1), KeyModifiers::NONE);
        assert_eq!(
            app.modal,
            Some(Modal::Help),
            "F1 should open help on {screen:?}"
        );

        press(&mut app, KeyCode::F(1), KeyModifiers::NONE);
        assert!(app.modal.is_none(), "F1 should close help on {screen:?}");
    }

    press(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    press(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.modal.is_none(), "Esc should close help too");
}

#[tokio::test]
async fn esc_leaves_the_settings_screen() {
    let mut app = test_app();
    app.screen = AppScreen::WarpgateSettings;

    press(&mut app, KeyCode::Esc, KeyModifiers::NONE);

    assert_eq!(app.screen, AppScreen::Main);
}

/// Guards the regression where starting on the settings screen queued no fetch, so reaching the
/// main screen left it stuck on `loading` with nothing explaining why.
#[tokio::test]
async fn an_unconfigured_start_still_asks_for_targets() {
    let mut app = test_app();
    assert_eq!(
        app.screen,
        AppScreen::WarpgateSettings,
        "an empty config should open the settings screen"
    );

    app.queue_startup_events();

    let event = tokio::time::timeout(std::time::Duration::from_secs(5), app.events.next())
        .await
        .expect("startup should have queued an event")
        .expect("the event channel should be open");

    assert!(matches!(event, Event::App(AppEvent::RefreshTargets)));
}

/// Navigation keys no longer re-rank the list, so the clamp they rely on has to hold on its own:
/// `select_last` parks the selection past the end until something pulls it back.
#[tokio::test]
async fn navigation_keys_keep_the_highlight_inside_the_list() {
    let mut app = test_app();
    app.screen = AppScreen::Main;
    *app.data.warpgate_targets.lock().unwrap() =
        Ok(vec![target("alpha"), target("beta"), target("gamma")]);
    app.recalculate_filtered_targets();

    press(&mut app, KeyCode::End, KeyModifiers::NONE);
    assert_eq!(app.table_targets_selection_state.selected(), Some(2));

    for _ in 0..3 {
        press(&mut app, KeyCode::Down, KeyModifiers::NONE);
    }
    assert_eq!(
        app.table_targets_selection_state.selected(),
        Some(2),
        "Down at the bottom must not walk past the last row"
    );

    set_search_query(&mut app, "no-such-target");
    app.recalculate_filtered_targets();
    press(&mut app, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(
        app.table_targets_selection_state.selected(),
        None,
        "an empty list has nothing to highlight"
    );
}
