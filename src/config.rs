use std::sync::{Arc, Mutex};

use color_eyre::eyre::{Context, eyre};
use config::{File, FileFormat};
use serde::{Deserialize, Serialize};

pub const DEFAULT_WARPGATE_PORT: u16 = 2222;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppConfig {
    pub warpgate_api_url: Option<String>,
    pub warpgate_token: Option<String>,
    pub warpgate_username: Option<String>,
    pub warpgate_port: Option<u16>,
}

impl AppConfig {
    pub fn get_config_file_path() -> color_eyre::Result<std::path::PathBuf> {
        let project_dirs =
            directories::ProjectDirs::from("com", "warpgate-connect", "warpgate-connect")
                .ok_or_else(|| eyre!("Could not determine the user configuration directory"))?;

        Ok(project_dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> color_eyre::Result<Arc<Mutex<Self>>> {
        let config_path = Self::get_config_file_path()?;
        tracing::info!(path = %config_path.display(), "Loading configuration");

        let cfg = config::Config::builder()
            .set_default::<&str, Option<String>>("warpgate_api_url", None)?
            .set_default::<&str, Option<String>>("warpgate_token", None)?
            .set_default::<&str, Option<String>>("warpgate_username", None)?
            .set_default::<&str, Option<u16>>("warpgate_port", DEFAULT_WARPGATE_PORT.into())?
            .add_source(
                File::from(config_path.clone())
                    .required(false)
                    .format(FileFormat::Toml),
            )
            .build()?;

        let app_config = cfg.try_deserialize::<AppConfig>().with_context(|| {
            format!(
                "Failed to deserialize configuration at {}. Please check your config file for errors.",
                config_path.display()
            )
        })?;

        if !app_config.are_all_required_fields_set() {
            tracing::warn!(
                "Configuration loaded but some required fields are missing, saving current configuration with defaults"
            );
            app_config.save()?;
        } else {
            tracing::info!("Configuration loaded successfully");
        }

        Ok(Arc::new(Mutex::new(app_config)))
    }

    pub fn save(&self) -> color_eyre::Result<()> {
        let config_path = Self::get_config_file_path()?;
        tracing::info!(path = %config_path.display(), "Saving configuration");

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }

        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, toml_string)
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
