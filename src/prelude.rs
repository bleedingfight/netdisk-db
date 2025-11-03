//! 统一模块导出 - 简化外部调用
//! 
//! 使用示例：
//! ```rust
//! use netdisk_db::prelude::*;
//! // 现在可以直接使用所有公共类型和函数
//! ```

// 从lib.rs中重新导出所有模块的公共类型
pub use crate::{
    // Models
    models::config::{AppConfig, DatabaseConfig, MultiDatabaseConfig},
    models::database::{Database, FileRecord},
    
    // Views
    views::ui::{file_records_to_model, database_list_to_model, AppWindow},
    
    // Controllers
    controllers::handlers::*,
    
    // Services
    services::database::sqlite::SqliteDatabase,
    services::database_manager::DatabaseManager,
    
    // Utils
    utils::common::{format_file_size, get_timestamp},
};

// Re-export commonly used external crates
pub use anyhow::Result;