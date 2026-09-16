use self_update::backends::github::{Update, UpdateBuilder};

const REPO_OWNER: &str = "stax124";
const REPO_NAME: &str = "warpgate-connect";
const BIN_NAME: &str = "warpgate-connect";

fn updater() -> UpdateBuilder {
    let mut builder = Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(env!("CARGO_PKG_VERSION"));
    builder
}

/// Returns the version of a newer release, or `None` when up to date or the check failed.
///
/// Blocks; call from `spawn_blocking`.
pub fn check_for_newer_version() -> Option<String> {
    let release = match updater().build().and_then(|u| u.get_latest_release()) {
        Ok(release) => release,
        Err(e) => {
            tracing::warn!(error = %e, "Update check failed");
            return None;
        }
    };

    let current = env!("CARGO_PKG_VERSION");
    match self_update::version::bump_is_greater(current, &release.version) {
        Ok(true) => Some(release.version),
        Ok(false) => None,
        Err(e) => {
            tracing::warn!(error = %e, version = %release.version, "Could not compare release versions");
            None
        }
    }
}

/// Downloads and installs the latest release, returning the version now on disk.
///
/// Blocks; call from `spawn_blocking`.
pub fn perform_update() -> color_eyre::Result<self_update::Status> {
    let mut builder = updater();
    builder
        .show_download_progress(true)
        .no_confirm(true)
        .show_output(false);

    Ok(builder.build()?.update()?)
}
