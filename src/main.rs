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

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let config = config::AppConfig::load()?;
    let config_for_execute = config.clone();

    let data = app_data::Data::new();
    let data_for_execute = data.clone();

    let terminal = ratatui::init();
    let result = App::new(data, config).run(terminal).await;
    ratatui::restore();

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
