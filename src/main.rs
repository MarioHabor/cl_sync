pub mod cl_sync;
pub mod cli;
pub mod error;
pub mod operations;

use anyhow::Result;
use clap_complete::Shell;
use std::path::PathBuf;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

use crate::operations::toml;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    let log_level = if matches.get_flag("debug") {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    debug!("Debug mode enabled!");

    let nointer = matches.get_flag("non_interactive");
    if nointer {
        debug!("Running in non-interactive mode.");
    }

    if matches.get_flag("synchronise") {
        let parsed_toml = toml::TomlParser::new().await?;
        cl_sync::begin_sync(&parsed_toml).await?;
    }

    if let Some(path) = matches.get_one::<PathBuf>("upload") {
        let parsed_toml = toml::TomlParser::new().await?;
        cl_sync::begin_upload(&parsed_toml, path.to_path_buf(), nointer).await?;
    }

    if matches.get_flag("check") {
        let _ = cl_sync::check_last_update();
    }

    if let Some(generator) = matches.get_one::<Shell>("generator") {
        let mut cmd = cli::build_cli();
        eprintln!("Generating completion file for {generator}...");
        cli::print_completions(*generator, &mut cmd);
    }
    Ok(())
}
