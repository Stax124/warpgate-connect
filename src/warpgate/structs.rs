#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct WarpgateTargetGroup {
    pub name: String,
    pub id: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WarpgateTarget {
    pub description: Option<String>,
    pub group: Option<WarpgateTargetGroup>,
    pub kind: String,
    pub name: String,
}

impl WarpgateTarget {
    pub fn is_ssh(&self) -> bool {
        self.kind == "Ssh"
    }
}

#[derive(Debug, Clone)]
pub struct WarpgateFilterableTarget {
    pub warpgate_target: WarpgateTarget,
    pub filterable_name: String,
}

impl WarpgateFilterableTarget {
    pub fn new(warpgate_target: WarpgateTarget) -> Self {
        let filterable_name = format!(
            "{} ({})",
            warpgate_target.name,
            warpgate_target.description.as_deref().unwrap_or("")
        );
        Self {
            warpgate_target,
            filterable_name,
        }
    }
}

impl AsRef<str> for WarpgateFilterableTarget {
    fn as_ref(&self) -> &str {
        &self.filterable_name
    }
}

#[cfg(test)]
#[path = "structs_test.rs"]
mod tests;
