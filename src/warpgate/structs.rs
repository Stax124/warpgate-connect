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
