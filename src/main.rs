use clap::Parser;
use tokio::process;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tui_logger::TuiTracingSubscriberLayer;

use crate::app::{App, Connection, ConnectionType, Outcome};

mod app;
mod config;
mod event;
mod list;
mod screens;
mod theme;
mod update;
mod utils;
mod warpgate;

#[derive(Debug, clap::Parser)]
#[command(version)]
struct Args {
    #[arg(
        long,
        help = "Skip the update check and proceed directly to the application."
    )]
    skip_update: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Print the SSH targets and the username each one needs, tab separated.
    List,
}

async fn execute_connection(connection: &Connection) -> color_eyre::Result<()> {
    let (program, port_argument) = match connection.connection_type {
        ConnectionType::Ssh => ("ssh", "-p"),
        ConnectionType::Sftp => ("sftp", "-P"),
    };

    process::Command::new(program)
        .arg(port_argument)
        .arg(connection.port.to_string())
        .arg("-o")
        .arg(format!("User={}", connection.ssh_username))
        .arg(&connection.host)
        .spawn()?
        .wait()
        .await?;

    tracing::info!(target = %connection.target_name, connection_type = %connection.connection_type, "Session closed");
    println!("Session closed. Goodbye!");

    Ok(())
}

async fn async_main(skip_update: bool) -> color_eyre::Result<()> {
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting warpgate-connect"
    );

    let config = config::AppConfig::load()?;

    let terminal = ratatui::init();
    let outcome = App::new(config, skip_update).run(terminal).await;
    ratatui::restore();

    match outcome? {
        Outcome::Quit => println!("No target selected. Quitting without connecting."),
        Outcome::MissingConfiguration => println!(
            "Cannot connect: Missing required configuration fields. Please check your settings."
        ),
        Outcome::Update => update::run_update().await,
        Outcome::Connect(connection) => {
            tracing::info!(target = %connection.target_name, host = %connection.host, "Connecting to {} target", connection.connection_type);
            println!("Connecting to: '{}'", connection.target_name);
            println!("Connection type: '{}'", connection.connection_type);

            execute_connection(&connection).await?;
        }
    }

    Ok(())
}

fn run_tokio_main(args: Args) -> color_eyre::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            match args.command {
                Some(Command::List) => list::print_targets(config::AppConfig::load()?).await,
                None => async_main(args.skip_update).await,
            }
        })?;

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    color_eyre::install()?;

    tui_logger::init_logger(tui_logger::LevelFilter::Info)?;
    tui_logger::set_default_level(tui_logger::LevelFilter::Info);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(TuiTracingSubscriberLayer)
        .init();

    run_tokio_main(args)
}
