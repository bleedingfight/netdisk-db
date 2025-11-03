pub mod sqlite;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub modified_time: String,
    pub file_type: String,
}

pub trait Database: Send + Sync {
    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>>;
    fn init_database(&self) -> Result<()>;
}