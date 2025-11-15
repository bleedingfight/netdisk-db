//! 数据库模型 - 数据库抽象接口和数据结构
//!
//! 定义数据库操作的通用接口和文件记录数据结构

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 文件记录数据结构
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct FileRecord {
//     pub id: i64,
//     pub name: String,
//     pub path: String,
//     pub size: i64,
//     pub modified_time: String,
//     pub file_type: String,
// }

// /// 文件记录数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub size: u64,  // 改为u64类型以支持更大的文件大小
    pub etag: String,
    pub modified_time: i64,
    pub file_type: String,
    pub name: String,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct ItemRecord<T> {
//     pub last_update_time: i32,
//     pub message: String,
//     pub data: Option<T>,
// }
// impl<T> ItemRecord<T> {
//     pub fn new(code: i32, message: String, data: T, x_trace_id: String) -> Self {
//         ApiResponse {
//             last_update_time: i32,
//             message: message,
//             data: Some(data),
//         }
//     }
// }

/// 数据库操作通用接口
///
/// 实现此接口可以为不同的数据库提供支持
pub trait Database: Send + Sync {
    /// 搜索文件
    ///
    /// # Arguments
    /// * `query` - 搜索关键词，支持模糊匹配
    ///
    /// # Returns
    /// * `Result<Vec<FileRecord>>` - 搜索结果列表
    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>>;

    /// 搜索特定字段
    ///
    /// # Arguments
    /// * `field` - 要搜索的字段名
    /// * `query` - 搜索关键词
    ///
    /// # Returns
    /// * `Result<Vec<FileRecord>>` - 搜索结果列表
    fn search_field(&self, _field: &str, query: &str) -> Result<Vec<FileRecord>> {
        // 默认实现：忽略字段参数，使用普通搜索
        self.search_files(query)
    }

    /// 获取支持的搜索字段
    ///
    /// # Returns
    /// * `Vec<String>` - 支持的字段列表
    fn get_search_fields(&self) -> Vec<String> {
        vec!["name".to_string(), "path".to_string()]
    }

    /// 初始化数据库
    ///
    /// 创建必要的表结构和索引
    fn init_database(&self) -> Result<()>;
}
