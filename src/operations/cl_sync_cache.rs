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

impl ClCache {
    pub async fn new(parsed_toml: &TomlParser) -> Result<Self> {
        let _ = sys_ops::config_dir_exists().await;

        let data = Self::load_from_file(&mut parsed_toml.clone())
            .await
            .unwrap_or_default();
        // Get the CacheDir section and extract the path
        let cache_storage_path = match parsed_toml
            .get_section_from_toml(TomlSection::CacheDir)
            .await
        {
            Ok(TomlToParse::CacheDir(dir)) => dir,

            _ => panic!("Unexpected section type for CacheDir"),
        };
        Ok(ClCache {
            data: Arc::new(Mutex::new(HashMap::from(data))),
            cache_storage_path,
        })
    }

    pub async fn insert(&self, upload: ToUpload) {
        let mut data = self.data.lock().await; // Lock the HashMap
        data.insert(upload.file_path.to_string(), upload); // Perform the insertion
    }

    pub async fn remove(&self, key: &str) -> Option<ToUpload> {
        let mut data = self.data.lock().await; // Lock the HashMap
        data.remove(key) // Perform the removal
    }

    pub async fn get(&self, key: &str) -> Option<ToUpload> {
        let data = self.data.lock().await; // Lock the HashMap for reading
        data.get(key).cloned() // Return a cloned value to avoid borrowing issues
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
            let _ = Self::create_cache_file(&mut cache_path, parsed_toml).await;
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

    async fn create_cache_file(
        cache_storage_path: &mut String,
        parsed_toml: &mut TomlParser,
    ) -> Result<()> {
        if !Self::directory_exists(&cache_storage_path).await {
            println!("directory_exists no");
            let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
            let config_path = home_path.join(".config/cl_sync/cache.bin");

            parsed_toml.update_cache_dir().await?;
            *cache_storage_path = config_path.to_string_lossy().to_string();
        }
        let encoded: Vec<u8> = bincode::serialize("").unwrap();
        let mut file = File::create(cache_storage_path)
            .await
            .expect("Can not write cache to path");
        let _ = file.write_all(&encoded).await;
        Ok(())
    }

    async fn directory_exists(path: &str) -> bool {
        match fs::metadata(path).await {
            Ok(metadata) => metadata.is_dir(),
            Err(_) => false,
        }
    }

    pub async fn save_to_file(&self) -> Result<()> {
        // Lock the Mutex to access the data
        let data = self.data.lock().await;
        let encoded: Vec<u8> = bincode::serialize(&*data).unwrap();
        let mut file = File::create(&self.cache_storage_path)
            .await
            .expect("Can not write cache to path");
        let _ = file
            .write_all(&encoded)
            .await
            .expect("Can not write cache to path");

        Ok(())
    }
}

#[cfg(test)]
mod cache_test {
    use super::*;

    #[tokio::test]
    async fn test_get_home() {
        let _ = sys_ops::config_dir_exists().await;
    }
}

#[tokio::test]
async fn cache_testing() -> Result<()> {
    let parsed_toml = TomlParser::new().await?;

    let cache = ClCache::new(&parsed_toml).await?;

    if let Some(structure) = cache.get("/home/dev/Desktop/Master_Passworlds.kdbx").await {
        println!(
            "Found existing file: {}, last checked: {}",
            structure.file_path, structure.last_saved
        );
    } else {
        println!("Existing file not found in cache");
    }
    // Check a file
    let file_path = "/home/dev/Desktop/Master_Passworlds.kdbx";
    match cache.get(file_path).await {
        Some(info) => {
            println!("File {} was last checked at {}", file_path, info.last_saved);
            assert!(
                !info.last_saved.to_string().is_empty(),
                "Last saved data should not be empty"
            );
        }
        None => println!("File {} has never been checked before", file_path),
    }

    Ok(())
}
