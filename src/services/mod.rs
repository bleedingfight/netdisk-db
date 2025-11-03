//! 服务模块 - 核心业务逻辑实现
//!
//! 包含数据库服务、文件处理服务等核心业务逻辑

pub mod database;
pub mod database_manager;

// 重新导出常用类型
pub use database_manager::DatabaseManager;