use anyhow::{Ok, Result};
use std::path::PathBuf;
use tracing::debug;

use crate::operations::sys_ops;
use crate::toml;

pub async fn check_last_update() {
    todo!();
}

pub async fn begin_upload(parsed_toml: &toml::TomlParser, path: PathBuf) -> Result<()> {
    if sys_ops::is_dir(path).await {
        debug!("Uploading directory.");
    } else {
        debug!("Uploading file.");
    }

    Ok(())
}
