use clap::Parser;
use tokio::process;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tui_logger::TuiTracingSubscriberLayer;

use crate::{app::App, utils::get_domain_from_warpgate_url};

mod app;
mod app_data;
mod config;
mod event;
mod screens;
mod ui;
mod utils;
mod warpgate;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(
        long,
        help = "Skip the update check and proceed directly to the application."
    )]
    skip_update: bool,
}

async fn async_main(skip_update: bool) -> color_eyre::Result<()> {
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting warpgate-connect"
    );

    let config = config::AppConfig::load()?;
    let config_for_execute = config.clone();

    let data = app_data::Data::new();
    let data_for_execute = data.clone();

    let terminal = ratatui::init();
    let result = App::new(data, config, skip_update).run(terminal).await;
    ratatui::restore();

    // Handle update if the user triggered it
    if *data_for_execute.trigger_update.lock().unwrap() {
        tracing::info!("User triggered update, starting update process");
        println!("Starting update...");

        tokio::task::spawn_blocking(move || {
            let mut updater = self_update::backends::github::Update::configure();
            updater
                .repo_owner("stax124")
                .repo_name("warpgate-connect")
                .bin_name("warpgate-connect")
                .show_download_progress(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .no_confirm(true)
                .show_output(true);

            let auth_token = std::env::var("GITHUB_AUTH_TOKEN");
            if let Ok(ref token) = auth_token {
                updater.auth_token(token);
            }

            match updater.build().unwrap().update() {
                Ok(status) => {
                    if status.updated() {
                        tracing::info!(version = %status.version(), "Successfully updated");
                        println!(
                            "Updated to version {}. Please restart the application.",
                            status.version()
                        );
                    } else {
                        tracing::info!("Already up to date");
                        println!("Already up to date.");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Update failed");
                    println!("Update failed: {}", e);
                }
            }
        })
        .await?;

        return result;
    }

    let selected_target = data_for_execute.selected_target.lock().unwrap().clone();

    match selected_target {
        Some(target) => {
            let config = config_for_execute.lock().unwrap().clone();
            if !config.are_all_required_fields_set() {
                tracing::error!("Cannot connect: missing required configuration fields");
                println!(
                    "Cannot connect: Missing required configuration fields. Please check your settings."
                );
                return Ok(());
            }

            let domain = get_domain_from_warpgate_url(config.warpgate_api_url.as_ref().unwrap());
            if domain.is_none() {
                tracing::error!(url = ?config.warpgate_api_url, "Cannot connect: failed to extract domain from Warpgate API URL");
                println!("Cannot connect: Invalid Warpgate API URL.");
                return Ok(());
            }

            tracing::info!(target = %target.name, domain = ?domain, "Connecting to SSH target");
            println!("Connecting to: '{}'", target.name);

            process::Command::new("ssh")
                .arg("-p")
                .arg(config.warpgate_port.unwrap_or(2222).to_string())
                .arg("-o")
                .arg(format!(
                    "User={}:{}",
                    config.warpgate_username.as_ref().unwrap(),
                    target.name
                ))
                .arg(domain.unwrap())
                .spawn()?
                .wait()
                .await?;

            tracing::info!(target = %target.name, "SSH session closed");
            println!("Session closed. Goodbye!");
        }
        None => {
            tracing::debug!("No target selected, quitting without connecting");
            println!("No target selected. Quitting without connecting.");
        }
    }

    result
}

fn run_tokio_main(skip_update: bool) -> color_eyre::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main(skip_update).await })?;

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    let _ = dotenvy::dotenv();
    color_eyre::install()?;

    tui_logger::init_logger(tui_logger::LevelFilter::Info)?;
    tui_logger::set_default_level(tui_logger::LevelFilter::Info);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(TuiTracingSubscriberLayer)
        .init();

    run_tokio_main(args.skip_update)
}
