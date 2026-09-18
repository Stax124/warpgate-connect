#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct WarpgateTargetGroup {
    pub name: String,
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

#[cfg(test)]
#[path = "target_test.rs"]
mod tests;
