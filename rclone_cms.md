curl -X POST http://localhost:5574/sync/sync \
    -H "Content-Type: application/json" \
    -d '{ "srcFs": "/home/dev/Documents/OBvault2/", "dstFs": "dge:OBvault", "createEmptySrcDirs": true, "_async": true }'


params : {"srcFs": "/home/dev/Documents/OBvault/", "dstFs": "dge:OBvault", "createEmptySrcDirs": "true", "_async": "true"}

sync/copy: copy a directory from source remote to destination remote

This takes the following parameters:

    srcFs - a remote name string e.g. "drive:src" for the source
    dstFs - a remote name string e.g. "drive:dst" for the destination
    createEmptySrcDirs - create empty src directories on destination if set


curl -X POST http://localhost:5574/operations/copy \
    -H "Content-Type: application/json" \
    -d '{ 
          "srcFs": "/home/dev/Documents/OBvault2/", 
          "dstFs": "dge:/backup/", 
          "_async": true
        }'

curl -X POST http://localhost:5574/mount/mount \
    -H "Content-Type: application/json" \
    -d '{ "fs": "dge:", "mountPoint": "/home/dev/Documents/cloud/dge", "_async": true }'

 curl -X POST "http://localhost:5574/job/status" \
     -H "Content-Type: application/json" \
     -d '{ "jobid": "1" }'
{

curl -X POST "http://localhost:5574/job/status" \
     -H "Content-Type: application/json" \
     -d '{ "jobid": 17 }'
{
        "duration": 1.248352007,
        "endTime": "2025-02-15T19:36:43.64481602Z",
        "error": "",
        "finished": true,
        "group": "job/17",
        "id": 17,
        "output": {},
        "startTime": "2025-02-15T19:36:42.396464063Z",
        "success": true
}




async fn sync(parsed_toml: &toml::TomlParser) -> Result<()> {
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
        let _ = job_progress(&mut mount_jobid).await;

        let mut mount_jobid: Vec<u16> = vec![];
        for remote in &to_up.upload_to_clouds {
            let remote_path = format!("{}:{}", remote, to_up.upload_to_cloud_dir);
            let sync =
                rclone::sync_sync(to_up.file_or_dir_path.clone(), remote_path.to_string()).await?;
            mount_jobid.push(sync.job_id.unwrap());
        }
        let _ = job_progress(&mut mount_jobid).await;
        for remote in &to_up.upload_to_clouds {
            sys_ops::async_run_rclone_dismount(&remote_list.get(remote).unwrap().dir).await?;
        }
    }

    // Stop rclone when done
    rclone_server.stop().await;
    Ok(())
}



copyfile

curl -X POST http://localhost:5574/operations/copyfile \
    -H "Content-Type: application/json" \
    -d '{ "srcFs": "/home/dev/Documents/OBvault2/", "dstFs": "dge:OBvault", "createEmptySrcDirs": true, "_async": true }'

{
  "srcFs": "remote1:",
  "srcFile": "path/to/source-file.txt",
  "dstFs": "remote2:",
  "dstFile": "path/to/destination-file.txt"
}
