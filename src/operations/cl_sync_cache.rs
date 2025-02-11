use crate::operations::sys_ops;

use anyhow::Result;
use chrono::{DateTime, Local};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    pub async fn new() -> Result<Self> {
        let _ = sys_ops::config_dir_exists().await;
        todo!()
    }
}

#[tokio::test]
async fn test_get_home() {
    let _ = sys_ops::config_dir_exists().await;
}
