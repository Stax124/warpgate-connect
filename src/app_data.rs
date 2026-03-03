use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Data {
    pub warpgate_targets:
        Arc<Mutex<Result<Vec<crate::warpgate::structs::WarpgateTarget>, color_eyre::Report>>>,
    pub selected_target: Arc<Mutex<Option<crate::warpgate::structs::WarpgateTarget>>>,
    pub loading_targets: Arc<Mutex<bool>>,
    pub should_set_list_element_index: Arc<Mutex<bool>>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            warpgate_targets: Arc::new(Mutex::new(Ok(Vec::new()))),
            selected_target: Arc::new(Mutex::new(None)),
            loading_targets: Arc::new(Mutex::new(true)),
            should_set_list_element_index: Arc::new(Mutex::new(true)),
        }
    }
}
