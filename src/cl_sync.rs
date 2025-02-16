use anyhow::{anyhow, Result};
use dialoguer::Input;
use std::path::PathBuf;
use std::process::exit;
use std::u16;
use tracing::debug;

use crate::operations::rclone;
use crate::operations::sys_ops;
use crate::operations::toml;

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

pub async fn begin_sync(parsed_toml: &toml::TomlParser, nointe: bool) -> Result<()> {
    let mut rclone_server = rclone::RcloneServer::start().await;

    let upload_list = match parsed_toml
        .get_section_from_toml(toml::TomlSection::Upload)
        .await
    {
        Ok(toml::TomlToParse::Upload(dir)) => dir,
        _ => return Err(anyhow::anyhow!("Unexpected section type for upload list")),
    };
    let remote_list = match parsed_toml
        .get_section_from_toml(toml::TomlSection::CloudProviders)
        .await
    {
        Ok(toml::TomlToParse::CloudProviders(cloud)) => cloud,
        _ => return Err(anyhow::anyhow!("Unexpected section type for upload list")),
    };

    while !rclone::RcloneServer::is_running().await {
        println!("Waiting for rclone to start...");
    }

    for (_k, to_up) in &upload_list {
        // mount for this upload
        let mut mount_jobid: Vec<u16> = vec![];
        for remote in &to_up.upload_to_clouds {
            let mount = rclone::mount_remote(remote_list.get(remote).unwrap()).await?;
            mount_jobid.push(mount.job_id.unwrap());
        }
        while !mount_jobid.is_empty() {
            let mut completed_indices = vec![];

            // Collect indices of completed jobs first
            for (i, job_id) in mount_jobid.iter().enumerate() {
                if rclone::check_job_status(*job_id).await? {
                    completed_indices.push(i);
                }
            }

            // ✅ Remove completed jobs in reverse order to avoid shifting indices
            for &index in completed_indices.iter().rev() {
                mount_jobid.remove(index);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        debug!("{:?}", mount_jobid);

        let mut mount_jobid: Vec<u16> = vec![];
        for remote in &to_up.upload_to_clouds {
            let remote_path = format!("{}:{}", remote, to_up.upload_to_cloud_dir);
            let sync =
                rclone::sync_sync(to_up.file_or_dir_path.clone(), remote_path.to_string()).await?;
            mount_jobid.push(sync.job_id.unwrap());
        }
        while !mount_jobid.is_empty() {
            let mut completed_indices = vec![];

            for (i, job_id) in mount_jobid.iter().enumerate() {
                if rclone::check_job_status(*job_id).await? {
                    completed_indices.push(i);
                }
            }

            // Remove completed jobs in reverse order to avoid shifting indices
            for &index in completed_indices.iter().rev() {
                mount_jobid.remove(index);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        debug!("{:?}", mount_jobid);
    }

    // Stop rclone when done

    rclone_server.stop().await;

    Ok(())
}
