use color_eyre::eyre::eyre;

use crate::{
    config::{AppConfig, DEFAULT_WARPGATE_PORT},
    utils::{get_domain_from_warpgate_url, warpgate_ssh_username},
    warpgate::{fetch::fetch_configured_targets, target::WarpgateTarget},
};

pub async fn print_targets(config: AppConfig) -> color_eyre::Result<()> {
    let config_path = AppConfig::get_config_file_path()?;

    let warpgate_api_url = config.warpgate_api_url.as_deref().ok_or_else(|| {
        eyre!(
            "Warpgate API URL is not configured in {}",
            config_path.display()
        )
    })?;
    let warpgate_username = config.warpgate_username.as_deref().ok_or_else(|| {
        eyre!(
            "Warpgate username is not configured in {}",
            config_path.display()
        )
    })?;

    let host = get_domain_from_warpgate_url(warpgate_api_url)
        .ok_or_else(|| eyre!("Could not derive a hostname from {warpgate_api_url}"))?;

    let targets = fetch_configured_targets(&config).await?;

    print!(
        "{}",
        render_targets(
            &targets,
            warpgate_username,
            &host,
            config.warpgate_port.unwrap_or(DEFAULT_WARPGATE_PORT),
        )
    );

    Ok(())
}

/// Fields go out verbatim: a tab cannot occur in a Warpgate target name, so nothing needs quoting.
fn render_targets(
    targets: &[WarpgateTarget],
    warpgate_username: &str,
    host: &str,
    port: u16,
) -> String {
    let mut output = String::from("target\tusername\thost\tport\n");

    for target in targets.iter().filter(|target| target.is_ssh()) {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            target.name,
            warpgate_ssh_username(warpgate_username, &target.name),
            host,
            port
        ));
    }

    output
}

#[cfg(test)]
#[path = "list_test.rs"]
mod tests;
