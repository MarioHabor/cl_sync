use crate::operations::sys_ops;
use crate::operations::toml::{TomlParser, TomlSection, TomlToParse};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use hashbrown::HashMap;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

// This is the bincode file that gets loaded in to memory
pub struct ClCache {
    pub data: Arc<Mutex<HashMap<String, ToUpload>>>,
    pub cache_storage_path: String,
}

// This is each individual representation of files
// that need to be checked and uploaded
// file_path: where the file or dir lives
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToUpload {
    pub file_path: String,
    pub last_saved: DateTime<Local>,
}

#[allow(dead_code)]
impl ClCache {
    pub async fn new() -> Result<Self> {
        let _ = sys_ops::config_dir_exists().await;
        todo!()
    }

    async fn load_from_file(parsed_toml: &mut TomlParser) -> Result<HashMap<String, ToUpload>> {
        let mut cache_path = match parsed_toml
            .get_section_from_toml(TomlSection::CacheDir)
            .await
        {
            Ok(TomlToParse::CacheDir(dir)) => dir,
            Ok(_) => {
                return Err(anyhow!("Unexpected section returned for 'CacheDir'"));
                // Early return to handle unexpected cases
            }
            Err(err) => {
                return Err(err); // Early return to propagate the error
            }
        };
        if !Self::file_exists(&cache_path).await {
            parsed_toml.update_cache_dir().await?;
            Self::create_cache_file(&mut cache_path).await;
        }
        let mut file = File::open(cache_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        let data: HashMap<String, ToUpload> = bincode::deserialize(&buffer).unwrap();
        Ok(data)
    }

    async fn file_exists(path: &str) -> bool {
        match fs::metadata(path).await {
            Ok(metadata) => metadata.is_file(), // Check if the path is a file
            Err(_) => false,
        }
    }

    async fn create_cache_file(cache_storage_path: &mut String) {
        if !Self::directory_exists(&cache_storage_path).await {
            let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
            let config_path = home_path.join(".config/cl_sync/cache.bin");
            // add more logic to check if default directory exists

            *cache_storage_path = config_path.to_string_lossy().to_string();
        }
        let encoded: Vec<u8> = bincode::serialize("").unwrap();
        let mut file = File::create(cache_storage_path)
            .await
            .expect("Can not write cache to path");
        let _ = file.write_all(&encoded).await;
    }

    #[allow(dead_code)]
    async fn directory_exists(path: &str) -> bool {
        match fs::metadata(path).await {
            Ok(metadata) => metadata.is_dir(),
            Err(_) => false,
        }
    }
}

#[tokio::test]
async fn test_get_home() {
    let _ = sys_ops::config_dir_exists().await;
}
