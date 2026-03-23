#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct WarpgateTargetGroup {
    pub name: String,
    pub id: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WarpgateTarget {
    pub description: Option<String>,
    pub external_host: Option<String>,
    pub group: Option<WarpgateTargetGroup>,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WarpgateFilterableTarget {
    pub warpgate_target: WarpgateTarget,
    pub filterable_name: String,
}

impl WarpgateFilterableTarget {
    pub fn new(warpgate_target: WarpgateTarget) -> Self {
        let filterable_name = format!(
            "{} ({})",
            warpgate_target.name,
            warpgate_target
                .description
                .as_ref()
                .unwrap_or(&"".to_string())
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
