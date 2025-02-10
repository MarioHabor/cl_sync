use anyhow::Result;
use async_recursion::async_recursion;
use std::path::Path;
use tokio::fs;

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

#[tokio::test]
async fn test_read_dir_content() {
    // Test dismounting
    let entries = Path::new("/home/dev/Documents/palyOB/OBvault/");
    assert!(read_dir_content(entries).await.is_ok());
}
