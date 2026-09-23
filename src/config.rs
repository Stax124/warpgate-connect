use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, eyre};
use serde::{Deserialize, Serialize};

pub const DEFAULT_WARPGATE_PORT: u16 = 2222;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppConfig {
    pub warpgate_api_url: Option<String>,
    pub warpgate_token: Option<String>,
    pub warpgate_username: Option<String>,
    pub warpgate_port: Option<u16>,

    #[serde(skip)]
    pub path: PathBuf,
}

impl AppConfig {
    pub fn get_config_file_path() -> color_eyre::Result<PathBuf> {
        let project_dirs =
            directories::ProjectDirs::from("com", "warpgate-connect", "warpgate-connect")
                .ok_or_else(|| eyre!("Could not determine the user configuration directory"))?;

        Ok(project_dirs.config_dir().join("config.toml"))
    }

    pub fn load(config_path: &Path) -> color_eyre::Result<Self> {
        tracing::info!(path = %config_path.display(), "Loading configuration");

        let text = match std::fs::read_to_string(config_path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                return Err(e).with_context(|| format!("Failed to read {}", config_path.display()));
            }
        };

        let mut app_config: AppConfig = toml::from_str(&text).with_context(|| {
            format!(
                "Failed to parse the configuration at {}. Please check your config file for errors.",
                config_path.display()
            )
        })?;
        app_config.path = config_path.to_path_buf();
        app_config
            .warpgate_port
            .get_or_insert(DEFAULT_WARPGATE_PORT);

        if !app_config.are_all_required_fields_set() {
            tracing::warn!(
                "Configuration loaded but some required fields are missing, saving current configuration with defaults"
            );
            app_config.save()?;
        } else {
            tracing::info!("Configuration loaded successfully");
        }

        Ok(app_config)
    }

    pub fn save(&self) -> color_eyre::Result<()> {
        let config_path = &self.path;
        tracing::info!(path = %config_path.display(), "Saving configuration");

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }

        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(config_path, toml_string)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;

        tracing::info!(path = %config_path.display(), "Configuration saved");
        Ok(())
    }

    pub fn are_all_required_fields_set(&self) -> bool {
        self.warpgate_api_url.is_some()
            && self.warpgate_token.is_some()
            && self.warpgate_username.is_some()
            && self.warpgate_port.is_some()
    }
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
