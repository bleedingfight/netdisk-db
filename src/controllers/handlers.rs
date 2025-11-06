//! 控制器处理模块 - 业务逻辑和事件处理
//! 
//! 包含所有用户交互和业务流程的处理函数

use crate::models::database::Database;
use crate::views::ui::{file_records_to_model, database_list_to_string_model, AppWindow, FileItem};
use crate::services::database_manager::DatabaseManager;
use slint::{ModelRc, VecModel};
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


/// 处理数据库切换请求
///
/// # Arguments
/// * `database_index` - 数据库索引，-1 表示刷新列表
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

    // 处理刷新列表请求
    if database_index == -1 {
        info!("Refreshing database list...");
        let mut manager = database_manager.lock().unwrap();
        match manager.refresh_database_list() {
            Ok(_) => {
                info!("Database list refreshed successfully");
                // 更新UI中的数据库列表
                let database_list = manager.get_database_list();
                let database_model = database_list_to_string_model(database_list);
                ui.set_available_databases(database_model);
                
                // 重置当前选择为第一个数据库
                if manager.get_database_list().len() > 0 {
                    ui.set_current_database_index(0);
                }
            }
            Err(e) => {
                error!("Failed to refresh database list: {}", e);
            }
        }
        return;
    }

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

/// 处理文件右键菜单请求
///
/// # Arguments
/// * `file_item` - 文件项
/// * `x` - 鼠标X坐标
/// * `y` - 鼠标Y坐标
/// * `ui` - UI 弱引用
pub fn handle_file_context_menu(
    file_item: FileItem,
    x: f32,
    y: f32,
    ui: &slint::Weak<AppWindow>,
) {
    info!("=== RIGHT CLICK DETECTED ===");
    info!("File: {}, Position: ({}, {})", file_item.name, x, y);
    
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => {
            error!("Failed to upgrade UI handle");
            return;
        }
    };

    info!("Context menu requested for file: {} at position ({}, {})",
          file_item.name, x, y);
    
    // 设置选中的文件项
    ui.set_selected_file_item(file_item);
    ui.set_context_menu_visible(true);
    ui.set_context_menu_x(x as f32);
    ui.set_context_menu_y(y as f32);
    
    info!("=== CONTEXT MENU SHOULD BE VISIBLE ===");
}

/// 处理打开文件请求
///
/// # Arguments
/// * `file_path` - 文件路径
pub fn handle_open_file(file_path: &str) {
    info!("Opening file: {}", file_path);
    
    // 使用系统默认程序打开文件
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "start", "", file_path])
            .spawn();
    }
    
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(file_path)
            .spawn();
    }
    
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(file_path)
            .spawn();
    }
}

/// 处理打开文件位置请求
///
/// # Arguments
/// * `file_path` - 文件路径
pub fn handle_open_file_location(file_path: &str) {
    info!("Opening file location for: {}", file_path);
    
    if let Some(parent_path) = std::path::Path::new(file_path).parent() {
        let parent_string = parent_path.to_string_lossy().to_string();
        
        // 使用系统文件管理器打开文件夹
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(&parent_string)
                .spawn();
        }
        
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&parent_string)
                .spawn();
        }
        
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&parent_string)
                .spawn();
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
    
    // 设置数据库列表 - 使用字符串模型供ComboBox使用
    let database_model = database_list_to_string_model(database_list);
    ui.set_available_databases(database_model);
    ui.set_current_database_index(current_index as i32);
    
    debug!("Initialized database selector with {} databases",
           manager.get_database_list().len());
}
