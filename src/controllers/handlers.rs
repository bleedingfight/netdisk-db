//! 控制器处理模块 - 业务逻辑和事件处理
//! 
//! 包含所有用户交互和业务流程的处理函数

use crate::models::database::Database;
use crate::views::ui::{file_records_to_model, database_list_to_model, AppWindow};
use crate::services::database_manager::DatabaseManager;
use slint::{ModelRc, VecModel};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

/// 文件下载处理函数（模拟实现）
/// 
/// # Arguments
/// * `_url` - 文件URL或路径
pub fn download_proc<T>(_url: T)
where
    T: AsRef<str>,
{
    debug!("Download proc started");
    std::thread::sleep(std::time::Duration::from_secs(10));
    debug!("Download proc finished");
}

/// 发送到 Aria2 处理函数（模拟实现）
/// 
/// # Arguments
/// * `_url` - 文件URL或路径
pub fn send_to_aria2<T>(_url: T)
where
    T: AsRef<str>,
{
    debug!("Send to Aria2 proc started");
    std::thread::sleep(std::time::Duration::from_secs(10));
    debug!("Send to Aria2 proc finished");
}

/// 处理搜索请求
/// 
/// # Arguments
/// * `query` - 搜索关键词
/// * `ui` - UI 弱引用
/// * `database` - 数据库实例
/// * `last_search_time` - 上次搜索时间（用于防抖）
/// * `search_delay` - 搜索延迟时间
pub fn handle_search_request(
    query: &str,
    ui: &slint::Weak<AppWindow>,
    database: Arc<Mutex<dyn Database>>,
    last_search_time: Arc<Mutex<Instant>>,
    search_delay: Duration,
) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    // 防抖检查
    let now = Instant::now();
    let mut last_time = last_search_time.lock().unwrap();

    if now.duration_since(*last_time) < search_delay {
        return;
    }

    *last_time = now;
    drop(last_time);

    // 空查询处理
    if query.trim().is_empty() {
        let file_items = ModelRc::new(VecModel::default());
        ui.set_file_items(file_items);
        return;
    }

    // 执行搜索
    let results = database.lock().unwrap().search_files(query);
    match results {
        Ok(results) => {
            debug!("Search returned {} results", results.len());
            let file_items = file_records_to_model(results);
            ui.set_file_items(file_items);
        }
        Err(e) => {
            error!("Search failed: {}", e);
            ui.set_file_items(ModelRc::new(VecModel::default()));
        }
    }
}

/// 处理上下文菜单请求
/// 
/// # Arguments
/// * `file_id` - 文件ID
/// * `x` - X坐标
/// * `y` - Y坐标
/// * `ui` - UI 弱引用
/// * `_database` - 数据库实例（预留参数）
pub fn handle_context_menu_requested(
    file_id: i32,
    x: f32,
    y: f32,
    ui: &slint::Weak<AppWindow>,
    _database: Arc<Mutex<dyn Database>>,
) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    debug!(
        "Context menu requested for file ID: {} at position ({}, {})",
        file_id, x, y
    );

    // 获取当前选中的文件信息
    let selected_item = ui.get_selected_file_item();
    let file_path = selected_item.path.to_string();
    let file_name = selected_item.name.to_string();

    debug!("Selected file: {} at path: {}", file_name, file_path);
}

/// 处理发送到服务器请求
/// 
/// # Arguments
/// * `ui` - UI 弱引用
pub fn handle_send_to_server(ui: &slint::Weak<AppWindow>) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    // 获取当前选中的文件信息
    let selected_item = ui.get_selected_file_item();
    let file_path = selected_item.path.to_string();
    let file_name = selected_item.name.to_string();

    debug!(
        "Sending file to server: {} from path: {}",
        file_name, file_path
    );

    // 检查文件是否存在
    if Path::new(&file_path).exists() {
        debug!("File exists and can be sent to server");
        // 这里可以添加实际的上传逻辑
    } else {
        error!("File not found: {}", file_path);
    }
}

/// 处理更新内容请求
/// 
/// # Arguments
/// * `ui` - UI 弱引用
pub fn handle_update_content(ui: &slint::Weak<AppWindow>) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    // 获取当前选中的文件信息
    let selected_item = ui.get_selected_file_item();
    let file_path = selected_item.path.to_string();
    let file_name = selected_item.name.to_string();

    debug!(
        "Updating file content: {} at path: {}",
        file_name, file_path
    );

    // 检查文件是否存在
    if Path::new(&file_path).exists() {
        debug!("File exists and can be updated");
        // 这里可以添加实际的文件更新逻辑
    } else {
        error!("File not found: {}", file_path);
    }
}

/// 处理打开文件位置请求
/// 
/// # Arguments
/// * `ui` - UI 弱引用
pub fn handle_open_location(ui: &slint::Weak<AppWindow>) {
    // 尝试升级为强引用
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    // 获取当前选中的文件
    let selected_item = ui.get_selected_file_item();
    let file_path = selected_item.path.to_string();

    debug!("Opening file location: {}", file_path);

    // 获取文件所在目录
    if let Some(parent_dir) = Path::new(&file_path).parent() {
        debug!("Opening directory: {:?}", parent_dir);

        // 实际打开目录的逻辑
        if let Err(e) = open_in_file_manager(parent_dir) {
            error!("Failed to open directory: {}", e);
        }
    } else {
        error!("No parent directory found for file: {}", file_path);
    }
}

/// 处理数据库切换请求
///
/// # Arguments
/// * `database_index` - 数据库索引
/// * `ui` - UI 弱引用
/// * `database_manager` - 数据库管理器
pub fn handle_database_changed(
    database_index: i32,
    ui: &slint::Weak<AppWindow>,
    database_manager: Arc<Mutex<DatabaseManager>>,
) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    let index = database_index as usize;
    
    // 切换数据库
    let mut manager = database_manager.lock().unwrap();
    match manager.switch_database(index) {
        Ok(_) => {
            info!("Successfully switched to database index: {}", index);
            
            // 清空搜索结果
            ui.set_file_items(ModelRc::new(VecModel::default()));
            ui.set_search_text("".into());
        }
        Err(e) => {
            error!("Failed to switch database: {}", e);
        }
    }
}

/// 初始化数据库选择器
///
/// # Arguments
/// * `ui` - UI 弱引用
/// * `database_manager` - 数据库管理器
pub fn initialize_database_selector(
    ui: &slint::Weak<AppWindow>,
    database_manager: Arc<Mutex<DatabaseManager>>,
) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    let manager = database_manager.lock().unwrap();
    let database_list = manager.get_database_list();
    let current_index = manager.get_current_database_index();
    
    // 设置数据库列表
    let database_model = database_list_to_model(database_list);
    ui.set_available_databases(database_model);
    ui.set_current_database_index(current_index as i32);
    
    debug!("Initialized database selector with {} databases",
           manager.get_database_list().len());
}

/// 跨平台打开文件所在目录
pub fn open_in_file_manager(path: &Path) -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("explorer").arg(path).spawn()?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(path).spawn()?;
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    } else {
        error!("Unsupported platform for opening file location");
    }
    Ok(())
}