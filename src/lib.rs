//! 文件搜索工具库 - 统一模块导出
//! 
//! 提供现代模块化的文件搜索功能

// 主prelude模块 - 简化外部调用
pub mod prelude;

// 模型层 - 数据结构和业务接口
pub mod models {
    pub mod config;
    pub mod database;
}

// 视图层 - UI展示和数据转换
pub mod views {
    pub mod ui;
}

// 控制器层 - 业务逻辑和事件处理
pub mod controllers {
    pub mod handlers;
}

// 服务层 - 具体实现
pub mod services {
    pub mod database {
        pub mod sqlite;
    }
    pub mod database_manager;
}

// 工具层 - 通用工具函数
pub mod utils {
    pub mod common;
}

// 重新导出主要类型以提供简洁的API
pub use models::{
    config::{AppConfig, DatabaseConfig},
    database::{Database, FileRecord},
};
