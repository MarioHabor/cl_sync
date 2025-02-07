pub mod cli;

use anyhow::Result;
use clap_complete::Shell;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    if matches.get_flag("upload") {
        if matches.get_flag("synchronous") {
            //let _ = operation::begin_upload();
        } else {
            //let parsed_toml = TomlParser::new().await?;
            //async_ope::async_begin_upload(&parsed_toml).await?;
        }
    }

    if matches.get_flag("check") {
        //let _ = operation::check_last_update();
    }

    if let Some(generator) = matches.get_one::<Shell>("generator") {
        let mut cmd = cli::build_cli();
        eprintln!("Generating completion file for {generator}...");
        cli::print_completions(*generator, &mut cmd);
    }
    Ok(())
}
