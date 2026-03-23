use ratatui::style::Color;

use crate::app::App;

pub fn get_color_from_group_color(group_color: &Option<String>) -> Color {
    match group_color.as_deref() {
        Some("Primary") => Color::Blue,
        Some("Danger") => Color::Red,
        Some("Warning") => Color::Yellow,
        Some("Success") => Color::Green,
        _ => Color::Gray,
    }
}

pub fn try_set_first_index(app: &mut App, is_loading: &bool) {
    let mut should_set_list_element_index_guard =
        app.data.should_set_list_element_index.lock().unwrap();

    if *should_set_list_element_index_guard && !*is_loading {
        let warpgate_targets = app.data.warpgate_targets.lock().unwrap();
        if let Ok(targets) = warpgate_targets.as_ref()
            && !targets.is_empty()
        {
            app.table_targets_selection_state.select(Some(0));
        }
        *should_set_list_element_index_guard = false;
    }
}

pub fn get_domain_from_warpgate_url(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"^https?://([^:/]+)").unwrap();
    let result = re
        .captures(url)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));

    if result.is_none() {
        tracing::warn!(url = %url, "Failed to extract domain from warpgate URL");
    }

    result
}
