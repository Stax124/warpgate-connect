use super::*;

fn target(name: &str, kind: &str, group: Option<&str>) -> WarpgateTarget {
    WarpgateTarget {
        description: Some(format!("{name} host")),
        group: group.map(|name| WarpgateTargetGroup {
            name: name.to_string(),
            color: None,
        }),
        kind: kind.to_string(),
        name: name.to_string(),
    }
}

fn sample_targets() -> Vec<WarpgateTarget> {
    vec![
        target("prod-db", "Ssh", Some("Production")),
        target("prod-web", "Ssh", Some("Production")),
        target("staging-db", "Ssh", Some("Staging")),
        target("admin-panel", "Http", Some("Production")),
        target("orphan", "Ssh", None),
    ]
}

fn names(matched: &[MatchedTarget]) -> Vec<&str> {
    matched.iter().map(|m| m.target.name.as_str()).collect()
}

fn name_indices(matched: &[MatchedTarget], name: &str) -> Vec<u32> {
    matched
        .iter()
        .find(|m| m.target.name == name)
        .unwrap_or_else(|| panic!("{name} should be in the result"))
        .name_indices
        .clone()
}

#[test]
fn empty_query_keeps_every_ssh_target() {
    let filtered = filter_targets(&sample_targets(), None, "");

    let mut got = names(&filtered);
    got.sort_unstable();
    assert_eq!(got, ["orphan", "prod-db", "prod-web", "staging-db"]);
}

#[test]
fn non_ssh_targets_are_never_offered() {
    let filtered = filter_targets(&sample_targets(), None, "admin");
    assert!(
        filtered.is_empty(),
        "an Http target must not be connectable, got {:?}",
        names(&filtered)
    );
}

#[test]
fn group_filter_excludes_other_groups_and_ungrouped_targets() {
    let group = WarpgateTargetGroup {
        name: "Production".to_string(),
        color: None,
    };

    let filtered = filter_targets(&sample_targets(), Some(&group), "");

    let mut got = names(&filtered);
    got.sort_unstable();
    assert_eq!(got, ["prod-db", "prod-web"]);
}

#[test]
fn query_ranks_the_best_match_first() {
    let filtered = filter_targets(&sample_targets(), None, "stagingdb");
    assert_eq!(names(&filtered).first(), Some(&"staging-db"));
}

#[test]
fn query_matches_against_the_description_too() {
    let targets = vec![target("node-7", "Ssh", None)];
    let filtered = filter_targets(&targets, None, "node-7 host");
    assert_eq!(names(&filtered), ["node-7"]);
}

#[test]
fn a_name_match_reports_the_positions_it_matched() {
    let targets = vec![
        target("prod-db", "Ssh", None),
        target("old-web", "Ssh", None),
    ];

    let filtered = filter_targets(&targets, None, "prod");
    assert_eq!(name_indices(&filtered, "prod-db"), [0, 1, 2, 3]);

    let filtered = filter_targets(&targets, None, "web");
    assert_eq!(name_indices(&filtered, "old-web"), [4, 5, 6]);
}

#[test]
fn a_description_only_match_highlights_nothing_in_the_name() {
    let targets = vec![target("node-7", "Ssh", None)];

    // `target()` builds the description as "<name> host", so "host" is in no target's name.
    let filtered = filter_targets(&targets, None, "host");
    assert_eq!(names(&filtered), ["node-7"]);
    assert!(name_indices(&filtered, "node-7").is_empty());
}

#[test]
fn an_empty_query_highlights_nothing() {
    let filtered = filter_targets(&sample_targets(), None, "");
    assert!(filtered.iter().all(|m| m.name_indices.is_empty()));
}

#[test]
fn indices_stay_within_a_multi_byte_name() {
    let targets = vec![target("züri-db", "Ssh", None)];

    let filtered = filter_targets(&targets, None, "db");
    let character_count = "züri-db".chars().count() as u32;
    let indices = name_indices(&filtered, "züri-db");

    assert!(
        !indices.is_empty(),
        "the query should have matched the name"
    );
    assert!(
        indices.iter().all(|i| *i < character_count),
        "{indices:?} is outside a name of {character_count} characters"
    );
}

#[test]
fn extracts_domain_from_usable_urls() {
    assert_eq!(
        get_domain_from_warpgate_url("https://warpgate.example.com/@warpgate/api/targets"),
        Some("warpgate.example.com".to_string())
    );
    assert_eq!(
        get_domain_from_warpgate_url("http://warpgate.example.com:8888/@warpgate/api/targets"),
        Some("warpgate.example.com".to_string())
    );
    assert_eq!(
        get_domain_from_warpgate_url("https://10.0.0.5"),
        Some("10.0.0.5".to_string())
    );
}

#[test]
fn rejects_urls_without_a_recognisable_scheme() {
    assert_eq!(get_domain_from_warpgate_url(""), None);
    assert_eq!(get_domain_from_warpgate_url("warpgate.example.com"), None);
    assert_eq!(
        get_domain_from_warpgate_url("ssh://warpgate.example.com"),
        None
    );

    // The scheme is matched case-sensitively; an uppercased URL is rejected rather than accepted.
    assert_eq!(
        get_domain_from_warpgate_url("HTTPS://warpgate.example.com"),
        None
    );
}

#[test]
fn userinfo_is_captured_as_part_of_the_host() {
    // Documents current behaviour: the pattern stops at `:` or `/`, so a `user@host` form is
    // handed to ssh verbatim rather than being stripped.
    assert_eq!(
        get_domain_from_warpgate_url("https://user@warpgate.example.com/"),
        Some("user@warpgate.example.com".to_string())
    );
}
