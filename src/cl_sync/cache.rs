use crate::operations::{cl_sync_cache, sys_ops, toml};

use anyhow::Result;
use chrono::{DateTime, Local};
use tokio::fs;

pub async fn load(parsed_toml: &toml::TomlParser) -> Result<cl_sync_cache::ClCache> {
    cl_sync_cache::ClCache::new(parsed_toml).await
}

pub async fn exists(file: Option<&cl_sync_cache::ToUpload>) -> bool {
    if let Some(_structure) = file {
        return true;
    }
    println!("Existing file not found in cache");
    false
}

pub async fn get_last_update_from_cache(
    file: Option<&cl_sync_cache::ToUpload>,
) -> Result<DateTime<Local>> {
    if let Some(structure) = file {
        return Ok(structure.last_saved);
    }
    return Err(anyhow::anyhow!("Failed to read cache last saved"));
}

pub async fn compare_last_update(cache_time: DateTime<Local>, file_time: &String) -> Result<bool> {
    match fs::metadata(file_time.as_str()).await {
        Ok(data) => {
            if let Ok(modified) = data.modified() {
                let dt_file = sys_ops::to_epoch(modified.into()).await;
                let dt_cache = sys_ops::to_epoch(cache_time.into()).await;

                if dt_file > dt_cache {
                    return Ok(true);
                }
            } else {
                return Err(anyhow::anyhow!("Failed to read file modified date"));
            }
        }
        Err(err) => {
            return Err(anyhow::anyhow!("Error: {:?}", err));
        }
    }
    Ok(false)
}

pub async fn save_last_update_to_cache(
    file_or_dir_path: &str,
    parsed_toml: &toml::TomlParser,
) -> Result<()> {
    let cache = load(parsed_toml).await?;

    // Add a new file to the cache
    let new_file = cl_sync_cache::ToUpload {
        file_path: file_or_dir_path.to_string(),
        last_saved: Local::now(),
    };
    cache.insert(new_file).await;

    // Save updated cache
    cache.save_to_file().await?;

    Ok(())
}
