use tempfile::TempDir;

use super::*;

#[test]
fn loading_a_missing_file_writes_defaults_to_the_given_path() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("nested").join("config.toml");

    let config = AppConfig::load(&config_path).unwrap();

    assert_eq!(config.path, config_path);
    assert_eq!(config.warpgate_port, Some(DEFAULT_WARPGATE_PORT));
    assert!(!config.are_all_required_fields_set());

    let written = AppConfig::load(&config_path).unwrap();
    assert_eq!(written.warpgate_port, Some(DEFAULT_WARPGATE_PORT));
    assert_eq!(written.warpgate_api_url, None);
}

#[test]
fn saved_config_loads_back_from_the_same_path() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");

    AppConfig {
        warpgate_api_url: Some("https://localhost:8888/@warpgate/api/targets".into()),
        warpgate_token: Some("secret-token".into()),
        warpgate_username: Some("admin".into()),
        warpgate_port: Some(2022),
        path: config_path.clone(),
    }
    .save()
    .unwrap();

    let config = AppConfig::load(&config_path).unwrap();

    assert_eq!(config.path, config_path);
    assert_eq!(
        config.warpgate_api_url.as_deref(),
        Some("https://localhost:8888/@warpgate/api/targets")
    );
    assert_eq!(config.warpgate_token.as_deref(), Some("secret-token"));
    assert_eq!(config.warpgate_username.as_deref(), Some("admin"));
    assert_eq!(config.warpgate_port, Some(2022));
}

#[test]
fn saving_a_config_without_a_path_fails_instead_of_touching_the_real_file() {
    assert!(AppConfig::default().save().is_err());
}
