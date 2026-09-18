use super::*;

/// A trimmed response from `/@warpgate/api/targets`, covering the field combinations the API
/// actually emits: a coloured group, an explicit null description, and a target with no group.
const TARGETS_PAYLOAD: &str = r#"[
    {
        "id": "8f1c0a1e-0000-4000-8000-000000000001",
        "name": "prod-db",
        "description": "Primary database",
        "kind": "Ssh",
        "external_host": "db.internal",
        "group": { "name": "Production", "id": "grp-1", "color": "Danger" },
        "options": { "kind": "Ssh" }
    },
    {
        "id": "8f1c0a1e-0000-4000-8000-000000000002",
        "name": "staging-web",
        "description": null,
        "kind": "Ssh",
        "external_host": null,
        "group": { "name": "Staging", "id": "grp-2", "color": null }
    },
    {
        "id": "8f1c0a1e-0000-4000-8000-000000000003",
        "name": "admin-panel",
        "description": "Warpgate admin UI",
        "kind": "Http",
        "group": null
    }
]"#;

#[test]
fn deserializes_api_payload() {
    let targets: Vec<WarpgateTarget> =
        serde_json::from_str(TARGETS_PAYLOAD).expect("payload should deserialize");

    assert_eq!(targets.len(), 3);

    let group = targets[0].group.as_ref().expect("first target has a group");
    assert_eq!(group.name, "Production");
    assert_eq!(group.color.as_deref(), Some("Danger"));
    assert_eq!(targets[0].description.as_deref(), Some("Primary database"));

    assert_eq!(targets[1].description, None);
    assert_eq!(targets[1].group.as_ref().unwrap().color, None);

    assert_eq!(targets[2].group, None);
    assert_eq!(targets[2].kind, "Http");
}
