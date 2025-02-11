use crate::error;

use anyhow::{anyhow, Context, Result};
use hashbrown::HashMap;
use home::home_dir;
use serde_derive::Deserialize;
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::fs;
use toml;

use super::sys_ops;

#[derive(Debug, Deserialize, Clone)]
pub struct TomlData {
    pub upload: HashMap<String, TomlUpload>,
    pub cache_dir: CacheDir,
    pub cloud_providers: HashMap<String, CloudProviders>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheDir {
    pub dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TomlUpload {
    pub file_or_dir_name: String,
    pub file_or_dir_path: String,
    pub upload_to_clouds: Vec<String>,
    pub upload_to_cloud_dir: String,
    pub veracrypt_mount_dir: Option<String>,
    pub veracrypt_file_name: Option<String>,
    pub veracrypt_volume_pw: Option<String>,
    pub veracrypt_user_pw: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudProviders {
    pub cloud_name: String,
    pub dir: String,
    pub paste_to_dir: String,
}

pub enum TomlSection {
    Upload,
    CloudProviders,
    CacheDir,
}

pub enum TomlToParse {
    Upload(HashMap<String, TomlUpload>),
    CloudProviders(HashMap<String, CloudProviders>),
    CacheDir(String),
}

#[derive(Clone)]
pub struct TomlParser {
    data: TomlData,
}

impl TomlParser {
    // Initializes the struct by parsing the TOML file once
    pub async fn new() -> Result<Self> {
        let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let config_path = home_path.join(".config/cl_sync/upload.toml");

        let toml_data = match fs::read_to_string(&config_path).await {
            Ok(data) => data,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                sys_ops::create_toml_file().await?;
                fs::read_to_string(&config_path).await.context(format!(
                    "Failed to read newly created TOML file: {}",
                    config_path.display()
                ))?
            }
            Err(e) => return Err(error::TomlError::FileReadError(e).into()),
        };

        let data = toml::from_str(&toml_data).context(format!(
            "Failed to parse upload.toml file at {}",
            config_path.display()
        ))?;
        Ok(Self { data })
    }

    pub async fn get_section_from_toml(&self, section: TomlSection) -> Result<TomlToParse> {
        // Match the requested section
        match section {
            TomlSection::Upload => {
                // Check if `upload` is valid
                if self.data.upload.is_empty() {
                    Err(anyhow!(
                        "The 'upload' section is missing or empty in the upload.toml file"
                    ))
                } else {
                    Ok(TomlToParse::Upload(self.data.upload.clone()))
                }
            }
            TomlSection::CloudProviders => {
                // Check if `cloud_providers` is valid
                if self.data.cloud_providers.is_empty() {
                    Err(anyhow!(
                        "The 'cloud_providers' section is missing or empty in the upload.toml file"
                    ))
                } else {
                    Ok(TomlToParse::CloudProviders(
                        self.data.cloud_providers.clone(),
                    ))
                }
            }
            TomlSection::CacheDir => {
                // Check if `cloud_providers` is valid
                if self.data.cache_dir.dir.is_empty() {
                    Err(anyhow!(
                        "The 'cache_dir.dir' section is missing or empty in the upload.toml file"
                    ))
                } else {
                    Ok(TomlToParse::CacheDir(self.data.cache_dir.dir.clone()))
                }
            }
        }
    }
}
