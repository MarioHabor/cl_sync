use anyhow::Result;
use async_recursion::async_recursion;
use home::home_dir;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

#[async_recursion]
pub async fn read_dir_content(dir: &Path) -> Result<()> {
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        println!("{:?}", path.is_file());
        if path.is_dir() {
            println!("path: {:?}: {}", entry.file_name(), entry.ino());
            read_dir_content(&path).await?;
        } else {
            println!("{:?}: {}", entry, entry.ino());
        }
    }
    Ok(())
}

pub async fn config_dir_exists() -> Result<()> {
    // Get home directory safely
    let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let config_path = home_path.join(".config/cl_sync");

    // Check if directory exists
    if config_path.exists() {
        debug!("Config dir found");
        return Ok(());
    }

    // Create the directory (including parents if necessary)
    fs::create_dir_all(&config_path).await?;

    debug!("Config directory created: {:?}", config_path);
    Ok(())
}

#[tokio::test]
async fn test_read_dir_content() {
    let entries = Path::new("/home/dev/Documents/palyOB/OBvault/");
    assert!(read_dir_content(entries).await.is_ok());
}
