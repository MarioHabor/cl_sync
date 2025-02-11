use crate::error;

use anyhow::{anyhow, Context, Result};
use hashbrown::HashMap;
use home::home_dir;
use serde_derive::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::fs;
use toml;
use tracing::debug;

use super::sys_ops;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TomlData {
    #[serde(default)]
    pub upload: HashMap<String, TomlUpload>,
    #[serde(default)]
    pub cache_dir: CacheDir,
    #[serde(default)]
    pub cloud_providers: HashMap<String, CloudProviders>,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct CacheDir {
    pub dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

    /// Updates the `[cache_dir] dir` value and writes back to `upload.toml`
    pub async fn update_cache_dir(&mut self) -> Result<()> {
        let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        let toml_path = home_path.join(".config/cl_sync/upload.toml");
        let cache_path = home_path.join(".config/cl_sync/cache.bin");
        // Update the `dir` field in memory
        self.data.cache_dir.dir = cache_path.to_string_lossy().to_string();

        // Convert the updated struct back to a TOML string
        let updated_toml =
            toml::to_string_pretty(&self.data).context("Failed to serialize updated TOML")?;

        // Write back the modified TOML to the file
        fs::write(&toml_path, updated_toml)
            .await
            .context("Failed to write updated TOML file")?;

        debug!(
            "Updated [cache_dir] dir to: {}",
            cache_path.to_string_lossy()
        );
        Ok(())
    }
}

#[cfg(test)]
mod toml_parse_test {
    use super::*;

    #[tokio::test]
    async fn test_toml_parsing() -> Result<()> {
        let parser = TomlParser::new().await?;

        // Extract the "upload" section
        match parser.get_section_from_toml(TomlSection::Upload).await {
            Ok(TomlToParse::Upload(upload_data)) => {
                println!("Upload data: {:?}", upload_data);
            }
            Ok(_) => {
                eprintln!("Unexpected section returned for 'Upload'");
            }
            Err(err) => {
                eprintln!("Error extracting upload section: {:?}", err);
            }
        }
        let section = TomlSection::CloudProviders;

        match parser.get_section_from_toml(section).await {
            Ok(TomlToParse::CloudProviders(cloud_data)) => {
                // Assert or verify the parsed data
                println!("Cloud Providers data: {:?}", cloud_data);
                assert!(
                    !cloud_data.is_empty(),
                    "Cloud providers data should not be empty"
                );
            }
            Ok(_) => panic!("Unexpected section returned"),
            Err(err) => panic!("Test failed with error: {:?}", err),
        }
        match parser.get_section_from_toml(TomlSection::CacheDir).await {
            Ok(TomlToParse::CacheDir(cloud_data)) => {
                // Assert or verify the parsed data
                println!("Cache dir data: {:?}", cloud_data);
                assert!(!cloud_data.is_empty(), "Cache dir data should not be empty");
            }
            Ok(_) => panic!("Unexpected section returned"),
            Err(err) => panic!("Test failed with error: {:?}", err),
        }

        Ok(())
    }
}
