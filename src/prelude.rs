//! 统一导入模块 - 简化外部调用
//!
//! 提供项目中常用的类型和函数的便捷导入

// 重新导出主要类型
pub use crate::{
    models::{
        config::{AppConfig, DatabaseConfig},
        database::{Database, FileRecord},
    },
    services::database_manager::DatabaseManager,
    views::ui::{file_records_to_model, database_list_to_string_model, AppWindow},
    controllers::handlers::{
        handle_search_request,
        handle_database_changed,
        initialize_database_selector,
    },
    utils::common::{get_timestamp, format_file_size},
};

// 重新导出错误处理类型
pub use anyhow::Result;

// 重新导出异步运行时
pub use tokio;