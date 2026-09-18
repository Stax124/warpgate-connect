use super::*;

/// A trimmed response from `/@warpgate/api/targets`, including a non-SSH target and a name holding
/// the punctuation a comma-separated format would have had to escape.
const TARGETS_PAYLOAD: &str = r#"[
    {
        "id": "8f1c0a1e-0000-4000-8000-000000000001",
        "name": "prod-db",
        "description": "Primary database",
        "kind": "Ssh",
        "group": { "name": "Production", "id": "grp-1", "color": "Danger" }
    },
    {
        "id": "8f1c0a1e-0000-4000-8000-000000000002",
        "name": "admin-panel",
        "description": "Warpgate admin UI",
        "kind": "Http",
        "group": null
    },
    {
        "id": "8f1c0a1e-0000-4000-8000-000000000003",
        "name": "db, replica \"west\"",
        "description": null,
        "kind": "Ssh",
        "group": null
    }
]"#;

#[test]
fn renders_ssh_targets_only() {
    let targets: Vec<WarpgateTarget> =
        serde_json::from_str(TARGETS_PAYLOAD).expect("payload should deserialize");

    let rendered = render_targets(&targets, "admin", "warpgate.example.com", 2222);

    assert_eq!(
        rendered,
        "target\tusername\thost\tport\n\
         prod-db\tadmin:prod-db\twarpgate.example.com\t2222\n\
         db, replica \"west\"\tadmin:db, replica \"west\"\twarpgate.example.com\t2222\n"
    );
    assert!(!rendered.contains("admin-panel"));
}

#[test]
fn renders_a_header_when_nothing_is_reachable_over_ssh() {
    let targets: Vec<WarpgateTarget> = serde_json::from_str(TARGETS_PAYLOAD).unwrap();
    let http_only: Vec<WarpgateTarget> = targets.into_iter().filter(|t| !t.is_ssh()).collect();

    assert_eq!(
        render_targets(&http_only, "admin", "warpgate.example.com", 2222),
        "target\tusername\thost\tport\n"
    );
}
