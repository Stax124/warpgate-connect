use clap::Parser;
use tokio::process;

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
    let config = config::AppConfig::load()?;
    let config_for_execute = config.clone();

    let data = app_data::Data::new();
    let data_for_execute = data.clone();

    let terminal = ratatui::init();
    let result = App::new(data, config, skip_update).run(terminal).await;
    ratatui::restore();

    // Handle update if the user triggered it
    if *data_for_execute.trigger_update.lock().unwrap() {
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
                        println!(
                            "Updated to version {}. Please restart the application.",
                            status.version()
                        );
                    } else {
                        println!("Already up to date.");
                    }
                }
                Err(e) => {
                    println!("Update failed: {}", e);
                }
            }
        })
        .await?;

        return result;
    }

    let selected_target = data_for_execute.selected_target.lock().unwrap();

    match &*selected_target {
        Some(target) => {
            let config = config_for_execute.lock().unwrap();
            if !config.are_all_required_fields_set() {
                println!(
                    "Cannot connect: Missing required configuration fields. Please check your settings."
                );
                return Ok(());
            }

            let domain = get_domain_from_warpgate_url(config.warpgate_api_url.as_ref().unwrap());
            if domain.is_none() {
                println!("Cannot connect: Invalid Warpgate API URL.");
                return Ok(());
            }

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

            println!("Session closed. Goodbye!");
        }
        None => println!("No target selected. Quitting without connecting."),
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

    run_tokio_main(args.skip_update)
}
