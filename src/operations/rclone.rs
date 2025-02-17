use chrono::{DateTime, Local};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tracing::debug;

use crate::operations::toml;

pub struct RcloneServer {
    pub process: Option<Child>,
}

impl RcloneServer {
    pub async fn start() -> Self {
        let process = Command::new("rclone")
            .arg("rcd")
            .arg("--rc-no-auth")
            .arg("--rc-addr=:5574")
            .arg("--rc-enable-metrics")
            .spawn()
            .expect("Failed to start rclone daemon");

        println!("rclone server started on port 5574");
        Self {
            process: Some(process),
        }
    }

    pub async fn is_running() -> bool {
        let client = Client::new();
        let url = "http://localhost:5574/metrics";

        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => true,
            _ => false,
        }
    }

    pub async fn stop(&mut self) {
        if let Some(child) = self.process.as_mut() {
            match child.kill().await {
                Ok(_) => println!("rclone server stopped"),
                Err(e) => eprintln!("Failed to stop rclone server: {}", e),
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneResponse {
    #[serde(rename = "jobid")]
    job_id: Option<u16>,
    duration: Option<f64>,
    error: Option<String>,
    finished: Option<bool>,
    group: Option<String>,
    id: Option<u16>,
    success: Option<bool>,

    #[serde(rename = "endTime")]
    end_time: Option<DateTime<Local>>,

    #[serde(rename = "startTime")]
    start_time: Option<DateTime<Local>>,
}

//        "duration": 1.248352007,
//"endTime": "2025-02-15T19:36:43.64481602Z",
//"error": "",
//"finished": true,
//"group": "job/17",
//"id": 17,
//"output": {},
//"startTime": "2025-02-15T19:36:42.396464063Z",
//"success": true

//#[derive(Debug, Serialize, Deserialize)]
//struct RcloneStatusResponse {}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct RcloneRquest {
    pub command: String,
    pub params: hashbrown::HashMap<String, String>,

    #[serde(rename = "jobid")]
    pub job_id: Option<u16>,

    pub finished: Option<bool>,
}

impl RcloneRquest {
    pub async fn post(&mut self) -> Result<RcloneResponse, reqwest::Error> {
        let client = Client::new();
        let command = format!("http://localhost:5574/{}", self.command);

        let response = client
            .post(command)
            .json(&self.params)
            .send()
            .await?
            .json::<RcloneResponse>()
            .await?;
        if response.job_id.is_some() {
            self.job_id = Some(response.job_id.unwrap());
        }
        if response.finished.is_some() {
            self.finished = Some(response.finished.unwrap());
        }
        println!("post response: \n{:?}", response);
        Ok(response)
    }
}

pub async fn sync_sync(from: String, upload_to: String) -> anyhow::Result<RcloneRquest> {
    let mut params = hashbrown::HashMap::new();
    params.insert("srcFs".to_string(), from);
    params.insert("dstFs".to_string(), upload_to);

    params.insert("createEmptySrcDirs".to_string(), "true".to_string());
    params.insert("_async".to_string(), "true".to_string());
    println!("params : {:?}", params);

    let mut rclone_rquest = RcloneRquest {
        command: "sync/sync".to_string(),
        params,
        job_id: None,
        finished: None,
    };
    rclone_rquest.post().await?;

    Ok(rclone_rquest)
}

pub async fn check_job_status(job_id: u16) -> anyhow::Result<bool> {
    let mut params = hashbrown::HashMap::new();
    params.insert("jobid".to_string(), job_id.to_string());

    //params.insert("_async".to_string(), "true".to_string());

    let mut rclone_rquest = RcloneRquest {
        command: "job/status".to_string(),
        params,
        job_id: None,
        finished: None,
    };
    rclone_rquest.post().await?;
    println!("cehck job stat \n{:?}", rclone_rquest);

    Ok(rclone_rquest.finished.unwrap())
}

pub async fn mount_remote(remote: &toml::CloudProviders) -> anyhow::Result<RcloneRquest> {
    let mut params = hashbrown::HashMap::new();
    //params.insert("fs".to_string(), "dge:".to_string());
    params.insert("fs".to_string(), remote.cloud_name.to_string());

    params.insert("mountPoint".to_string(), remote.dir.to_string());
    params.insert("_async".to_string(), "true".to_string());

    let mut rclone_rquest = RcloneRquest {
        command: "mount/mount".to_string(),
        params,
        job_id: None,
        finished: None,
    };
    rclone_rquest.post().await?;
    debug!("{:?} ", rclone_rquest);

    Ok(rclone_rquest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_rclone_server_start_stop() {
        let mut rclone_server = RcloneServer::start().await;

        // Simulate doing some work
        let start_time = Instant::now();
        sleep(Duration::from_secs(2)).await; // Reduced sleep for faster testing
        let elapsed = start_time.elapsed();

        assert!(
            elapsed >= Duration::from_secs(2),
            "Sleep did not last at least 2 seconds"
        );

        // Stop rclone when done
        rclone_server.stop().await;

        // Ensure process handle is dropped after stopping
        assert!(
            rclone_server.process.is_none()
                || rclone_server.process.as_mut().unwrap().id().is_none(),
            "Process should be stopped"
        );
    }

    #[tokio::test]
    async fn test_rclone_sync_sync_stop() {
        let mut rclone_server = RcloneServer::start().await;
        //sleep(Duration::from_secs(5)).await; // Adjust if needed

        while !RcloneServer::is_running().await {
            println!("Waiting for rclone to start...");
        }

        // Stop rclone when done
        //if let Err(e) = mount_remote().await {
        //    panic!("mount_remote() failed: {:?}", e);
        //}

        if let Err(e) = sync_sync(
            "/home/user/Documents/dir/".to_string(),
            "remote:dir".to_string(),
        )
        .await
        {
            panic!("mount_remote() failed: {:?}", e);
        }

        rclone_server.stop().await;

        // Ensure process handle is dropped after stopping
        assert!(
            rclone_server.process.is_none()
                || rclone_server.process.as_mut().unwrap().id().is_none(),
            "Process should be stopped"
        );
    }
}
