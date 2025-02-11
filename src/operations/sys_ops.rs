use anyhow::Result;
use async_recursion::async_recursion;
use home::home_dir;
use indoc::indoc;
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

pub async fn create_toml_file() -> Result<()> {
    // Get home directory safely
    let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let config_path = home_path.join(".config/cl_sync/upload.toml");

    let default_content = get_default_toml();
    fs::write(&config_path, default_content).await?;
    debug!("TOML file created: {:?}", config_path);
    Ok(())
}

fn get_default_toml() -> String {
    let home_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let config_path = home_path.join(".config/cl_sync");

    format!(
        indoc! {
        r#"
        [upload]
  [upload.txt]
#   Add the name of the file or dir
  file_or_dir_name = "Text File (4).txt"
#   Add the dir or the file you want to upload
  file_or_dir_path = "/home/user/Desktop/Text File (4).txt"
#   based on cloud_providers section bellow add one or more cloud_name 's
  upload_to_clouds = [ "dge", "ode_rcl" ]
#   cloud dir to upload to
  upload_to_cloud_dir = "OBvault"
#   optional Veracrypt container
  # veracrypt_mount_dir = "/home/user/Downloads/text-master"
  # veracrypt_file_name = "text-master"
  # veracrypt_volume_pw = "12345"

[cache_dir]
# dir = "cache.bin"
dir = "{}/cache.bin" 

# modify
[cloud_providers]
  [cloud_providers.dg]
  cloud_name = "dg"
  dir = "/home/user/Documents/cloud/dg/" 
  paste_to_dir = "dg:files/"

  [cloud_providers.od_rcl]
  cloud_name = "od_rcl"
  dir = "/home/user/Documents/cloud/od/" 
  paste_to_dir = "od_rcl:desk/"

  [cloud_providers.dge]
  cloud_name = "dge"
  dir = "/home/user/Documents/cloud/dge/" 
  paste_to_dir = "dge:desk/"

  [cloud_providers.ode_rcl]
  cloud_name = "ode_rcl"
  dir = "/home/user/Documents/cloud/ode/" 
  paste_to_dir = "ode_rcl:desk/"
        "#},
        config_path.to_string_lossy()
    )
}

#[tokio::test]
async fn test_read_dir_content() {
    let entries = Path::new("/home/dev/Documents/palyOB/OBvault/");
    assert!(read_dir_content(entries).await.is_ok());
}
