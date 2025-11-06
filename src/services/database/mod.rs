//! 数据库服务模块 - 数据库相关服务实现
//!
//! 包含不同数据库类型的具体实现和连接器

pub mod connector;
pub mod sqlite;

// 重新导出常用类型
pub use connector::{DatabaseConnector, DatabaseConnectorFactory, DatabaseConnectionInfo};
pub use sqlite::SqliteDatabase;