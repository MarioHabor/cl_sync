use anyhow::{anyhow, Result};
use dialoguer::Input;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use tracing::debug;

use crate::operations::rclone;
use crate::operations::rclone::RcloneServer;
use crate::operations::sys_ops;
use crate::operations::toml;

pub mod cache;

pub async fn check_last_update() {
    todo!();
}

// if mode is set to interactive
// interactive mode is set to true by default pass --nointe to disable
pub async fn interactive_mode_to_up(nointe: bool, path: &PathBuf) -> Result<bool> {
    if !nointe {
        let dir_or_file = if sys_ops::is_dir(path.to_path_buf()).await? {
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
    _parsed_toml: &toml::TomlParser,
    path: PathBuf,
    nointe: bool,
) -> Result<()> {
    let reupload_again = interactive_mode_to_up(nointe, &path).await.unwrap();
    debug!("Reupload again: {reupload_again}");
    if !reupload_again {
        // call sync opperation
    }

    if sys_ops::is_dir(path).await? {
        debug!("Uploading directory.");
    } else {
        debug!("Uploading file.");
    }

    Ok(())
}

pub async fn begin_sync(parsed_toml: &toml::TomlParser) -> Result<()> {
    let cache = cache::load(&parsed_toml).await.unwrap();
    let mut rclone_server: Option<RcloneServer> = None;

    let upload_list = match parsed_toml
        .get_section_from_toml(toml::TomlSection::Upload)
        .await
    {
        Ok(toml::TomlToParse::Upload(dir)) => dir,
        _ => return Err(anyhow::anyhow!("Unexpected section type for upload list")),
    };

    let mut mounted_remotes: Vec<String> = vec![];

    let mut is_rclone_server_started: bool = false;
    for (_k, to_up) in &upload_list {
        if cache::exists(cache.get(&to_up.file_or_dir_path).await.as_ref()).await {
            let cache_last_up = cache::get_last_update_from_cache(
                cache.get(&to_up.file_or_dir_path).await.as_ref(),
            )
            .await
            .unwrap();

            if cache::compare_last_update(cache_last_up, &to_up.file_or_dir_path).await? {
                let path = PathBuf::from(&to_up.file_or_dir_path);
                if sys_ops::is_dir(path.clone()).await? {
                    let server = sync(
                        parsed_toml,
                        is_rclone_server_started,
                        to_up,
                        &mut mounted_remotes,
                    )
                    .await?;
                    is_rclone_server_started = server.1;
                    rclone_server = Some(server.0);
                    let _ = cache::save_last_update_to_cache(&to_up.file_or_dir_path, &parsed_toml)
                        .await?;
                } else if sys_ops::is_file(path).await? {
                    let server = file_sync(
                        parsed_toml,
                        is_rclone_server_started,
                        to_up,
                        &mut mounted_remotes,
                    )
                    .await?;
                    is_rclone_server_started = server.1;
                    rclone_server = Some(server.0);
                    let _ = cache::save_last_update_to_cache(&to_up.file_or_dir_path, &parsed_toml)
                        .await?;
                } else {
                    eprintln!(
                        "Skipping: {} is neither a directory nor a file",
                        to_up.file_or_dir_path
                    );
                    continue;
                }
            }
            // new file to upload
        } else {
            let path = PathBuf::from(&to_up.file_or_dir_path);
            if sys_ops::is_dir(path.clone()).await? {
                let server = sync(
                    parsed_toml,
                    is_rclone_server_started,
                    to_up,
                    &mut mounted_remotes,
                )
                .await?;
                is_rclone_server_started = server.1;
                rclone_server = Some(server.0);
                let _ =
                    cache::save_last_update_to_cache(&to_up.file_or_dir_path, &parsed_toml).await?;
            } else if sys_ops::is_file(path).await? {
                let server = file_sync(
                    parsed_toml,
                    is_rclone_server_started,
                    to_up,
                    &mut mounted_remotes,
                )
                .await?;
                is_rclone_server_started = server.1;
                rclone_server = Some(server.0);
                let _ =
                    cache::save_last_update_to_cache(&to_up.file_or_dir_path, &parsed_toml).await?;
            } else {
                eprintln!(
                    "Skipping: {} is neither a directory nor a file",
                    to_up.file_or_dir_path
                );
                continue;
            }
        }
    }

    let _ = dismount_remotes(&mut mounted_remotes).await?;
    // Stop rclone when done
    if let Some(mut server) = rclone_server {
        server.stop().await;
    }
    Ok(())
}

async fn sync<'a>(
    parsed_toml: &'a toml::TomlParser,
    is_rclone_server_started: bool,
    to_up: &'a toml::TomlUpload,
    mounted_remotes: &'a mut Vec<String>,
) -> Result<(RcloneServer, bool, &'a mut Vec<String>)> {
    let rclone_server;

    if !is_rclone_server_started {
        rclone_server = RcloneServer::start().await;
    } else {
        //debug!("server all ready started.")
        println!("server all ready started.")
    }

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

    // mount for this upload
    let mut mount_jobid: Vec<u16> = vec![];
    for remote in &to_up.upload_to_clouds {
        let remote_path = remote_list.get(remote).unwrap();
        mounted_remotes.push(remote_path.dir.to_string());
        let mount = rclone::mount_remote(&remote_path).await?;
        mount_jobid.push(mount.job_id.unwrap());
    }
    let _ = job_progress(&mut mount_jobid).await;

    let mut mount_jobid: Vec<u16> = vec![];
    for remote in &to_up.upload_to_clouds {
        let remote_path = format!("{}:{}", remote, to_up.upload_to_cloud_dir);
        let sync =
            rclone::sync_sync(to_up.file_or_dir_path.clone(), remote_path.to_string()).await?;
        mount_jobid.push(sync.job_id.unwrap());
    }
    let _ = job_progress(&mut mount_jobid).await;

    // Stop rclone when done
    Ok((rclone_server, true, mounted_remotes))
}

async fn file_sync<'a>(
    parsed_toml: &'a toml::TomlParser,
    is_rclone_server_started: bool,
    to_up: &'a toml::TomlUpload,
    mounted_remotes: &'a mut Vec<String>,
) -> Result<(RcloneServer, bool, &'a mut Vec<String>)> {
    let rclone_server;

    if !is_rclone_server_started {
        rclone_server = RcloneServer::start().await;
    } else {
        //debug!("server all ready started.")
        println!("server all ready started.");
    }

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

    // mount for this upload
    let mut mount_jobid: Vec<u16> = vec![];
    for remote in &to_up.upload_to_clouds {
        let remote_path = remote_list.get(remote).unwrap();
        mounted_remotes.push(remote_path.dir.to_string());
        let mount = rclone::mount_remote(&remote_path).await?;
        mount_jobid.push(mount.job_id.unwrap());
    }
    let _ = job_progress(&mut mount_jobid).await;

    let mut mount_jobid: Vec<u16> = vec![];
    for remote in &to_up.upload_to_clouds {
        let local_dir = Path::new(&to_up.upload_to_cloud_dir);
        let colon_remote = format!("{}:", remote);
        let remote_dst_path = format!("{}:{}", remote, &to_up.upload_to_cloud_dir);

        let sync = rclone::copyfile(
            local_dir.parent().unwrap().to_string_lossy().to_string(),
            to_up.file_or_dir_name.to_string(),
            colon_remote,
            remote_dst_path,
        )
        .await?;
        mount_jobid.push(sync.job_id.unwrap());
    }
    let _ = job_progress(&mut mount_jobid).await;

    // Stop rclone when done
    Ok((rclone_server, true, mounted_remotes))
}

pub async fn job_progress(mount_jobid: &mut Vec<u16>) -> Result<()> {
    while !mount_jobid.is_empty() {
        mount_jobid.retain_mut(|job_id| {
            let status = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(rclone::check_job_status(*job_id))
            });

            match status {
                Ok(false) => true, // Keep the job if it's not completed
                Ok(true) => {
                    debug!("job_id {:?}", job_id);
                    false // Remove completed job
                }
                Err(e) => {
                    debug!("Error checking job status: {:?}", e);
                    true // Keep the job to retry
                }
            }
        });

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn dismount_remotes(mounted_remotes: &mut Vec<String>) -> Result<()> {
    //println!("{:?}", mounted_remotes);
    for remote in mounted_remotes {
        sys_ops::fusermount(remote).await?;
    }
    Ok(())
}
