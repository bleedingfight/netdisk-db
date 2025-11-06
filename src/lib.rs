//! 文件搜索工具库 - 统一模块导出
//!
//! 提供现代模块化的文件搜索功能

// 主prelude模块 - 简化外部调用
pub mod prelude;

// 直接模块声明 - 使用最新的Rust模块实现方式
pub mod models {
    pub mod config;
    pub mod database;
}

pub mod views {
    pub mod ui;
}

pub mod controllers {
    pub mod handlers;
    pub mod search_handler;
}

pub mod services {
    pub mod database_manager;
    pub mod database {
        pub mod connector;
        pub mod sqlite;
    }
}

pub mod utils {
    pub mod common;
}

// 重新导出主要类型以提供简洁的API
pub use models::config::{AppConfig, DatabaseConfig};
pub use models::database::{Database, FileRecord};

// 重新导出控制器函数
pub use controllers::handlers::{
    handle_search_request,
    handle_database_changed,
    handle_file_context_menu,
    handle_open_file,
    handle_open_file_location,
    initialize_database_selector,
};

// 重新导出服务类型
pub use services::database_manager::DatabaseManager;
