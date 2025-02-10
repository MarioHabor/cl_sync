use crate::operations::sys_ops;
use tokio::fs;

pub struct ClCache {}

impl ClCache {
    pub async fn new() -> Self {
        todo!()
    }
}

#[tokio::test]
async fn test_get_home() {
    let _ = sys_ops::config_dir_exists().await;
}
