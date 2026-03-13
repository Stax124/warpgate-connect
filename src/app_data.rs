use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Data {
    pub warpgate_targets:
        Arc<Mutex<Result<Vec<crate::warpgate::structs::WarpgateTarget>, color_eyre::Report>>>,
    pub selected_target: Arc<Mutex<Option<crate::warpgate::structs::WarpgateTarget>>>,
    pub loading_targets: Arc<Mutex<bool>>,
    pub should_set_list_element_index: Arc<Mutex<bool>>,
    /// Stores the latest available version string when an update is available.
    pub update_available: Arc<Mutex<Option<String>>>,
    /// Signals that the user wants to perform an update after TUI exit.
    pub trigger_update: Arc<Mutex<bool>>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            warpgate_targets: Arc::new(Mutex::new(Ok(Vec::new()))),
            selected_target: Arc::new(Mutex::new(None)),
            loading_targets: Arc::new(Mutex::new(true)),
            should_set_list_element_index: Arc::new(Mutex::new(true)),
            update_available: Arc::new(Mutex::new(None)),
            trigger_update: Arc::new(Mutex::new(false)),
        }
    }
}
