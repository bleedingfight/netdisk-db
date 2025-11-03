//! 控制器模块 - 业务逻辑和事件处理
//!
//! 包含所有用户交互和业务流程的处理函数

pub mod handlers;
pub mod search_handler;

// 重新导出常用函数
pub use handlers::*;
pub use search_handler::*;