mod sync;
mod upload;

use anyhow::{anyhow, Ok, Result};
use dialoguer::Input;
use std::path::PathBuf;
use std::process::exit;
use tracing::debug;

use crate::operations::sys_ops;
use crate::toml;

pub async fn check_last_update() {
    todo!();
}

// if mode is set to interactive
// interactive mode is set to true by default pass --nointe to disable
pub async fn interactive_mode_to_up(nointe: bool, path: &PathBuf) -> Result<bool> {
    if !nointe {
        let dir_or_file = if sys_ops::is_dir(path.to_path_buf()).await {
            "directory"
        } else {
            "file"
        };

        let over = Input::new()
            .with_prompt(format!(
        "Current {} exists.\nDo you want to upload everything again or sync modified files? (y|1 / n|0 / quit|q)",
        dir_or_file
    ))
            .validate_with(|input: &String| match input.to_lowercase().as_str() {
                "q" | "quit" => {
                    println!("Exiting.");
                    exit(0)
                }
                "yes" | "y" | "true" | "1" | "no" | "n" | "false" | "0" => Ok(()),
                _ => Err(anyhow!("Please enter 'yes/1' or 'no/0'")),
            })
            .interact_text()
            .unwrap()
            .to_lowercase();
        let over = matches!(over.as_str(), "yes" | "y" | "true" | "1");
        return Ok(over);
    }
    return Ok(false);
}

pub async fn begin_upload(
    parsed_toml: &toml::TomlParser,
    path: PathBuf,
    nointe: bool,
) -> Result<()> {
    let reupload_again = interactive_mode_to_up(nointe, &path).await.unwrap();
    debug!("Reupload again: {reupload_again}");
    if !reupload_again {
        // call sync opperation
    }

    if sys_ops::is_dir(path).await {
        debug!("Uploading directory.");
    } else {
        debug!("Uploading file.");
    }

    Ok(())
}
