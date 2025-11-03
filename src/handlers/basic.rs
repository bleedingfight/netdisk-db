use crate::database::{Database, FileRecord};
use netdisk_db::ui::ui_handler::*;
use slint::{ModelRc, VecModel};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error};

pub struct Downloader;
impl Downloader {
    pub fn new() -> Self {
        Downloader {}
    }
    pub fn download(&self, url: &str) {
        // Placeholder implementation
        debug!("Starting download from URL: {}", url);
    }
}
pub fn download_proc<T>(url: T)
where
    T: AsRef<str>,
{
    debug!("Download proc started");
    std::thread::sleep(std::time::Duration::from_secs(10));
    // debug!("Download proc called with URL: {}", url);
    debug!("Download proc finished");
}
pub fn open_file_locate<T>(url: T)
where
    T: AsRef<str>,
{
    debug!("Open file locate started");
    std::thread::sleep(std::time::Duration::from_secs(2));
    // debug!("Open file locate called with URL: {}", url);
    debug!("Open file locate finished");
}

pub fn send_to_aria2<T>(url: T)
where
    T: AsRef<str>,
{
    debug!("Download proc started");
    std::thread::sleep(std::time::Duration::from_secs(10));
    // debug!("Download proc called with URL: {}", url);
    debug!("Download proc finished");
}

pub fn handle_search_request(
    query: &str,
    ui: &slint::Weak<AppWindow>, // 或者你的具体 UI 类型
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

    if query.trim().is_empty() {
        let file_items = ModelRc::new(VecModel::default());
        ui.set_file_items(file_items);
        return;
    }

    // 执行搜索
    let results = database.lock().unwrap().search_files(query);
    match results {
        Ok(results) => {
            let file_items = file_records_to_model(results);
            ui.set_file_items(file_items);
        }
        Err(e) => {
            error!("Search failed: {}", e);
            ui.set_file_items(ModelRc::new(VecModel::default()));
        }
    }
}

pub fn handle_context_menu_requested(
    file_id: i32,
    x: f32,
    y: f32,
    ui: &slint::Weak<AppWindow>, // 假设你的 UI 类型是 AppWindow
    database: Arc<Mutex<dyn Database>>,
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

    // 这里可以根据需要添加更多的上下文菜单逻辑
    // 比如，根据文件类型启用/禁用菜单项，显示具体操作等
}

pub fn handle_send_to_server(ui: &slint::Weak<AppWindow>, // 传递 UI 句柄
) {
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

        // 这里添加实际的上传逻辑，比如通过 HTTP POST 请求上传文件
        // 你可以使用 reqwest 或其他 HTTP 客户端库来实现这个功能。
    } else {
        error!("File not found: {}", file_path);
    }
}

pub fn handle_update_content(ui: &slint::Weak<AppWindow>, // 传递 UI 句柄
) {
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

        // 这里添加实际的文件更新逻辑，比如打开文件编辑器、加载文件内容等
        // 比如，使用某个库打开文件进行编辑，或者将文件内容加载到 UI 中显示等
    } else {
        error!("File not found: {}", file_path);
    }
}

pub fn handle_open_location(ui: &slint::Weak<AppWindow>, // 弱引用 UI 句柄
) {
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

        // 实际打开目录的逻辑：
        // 根据不同平台使用不同命令
        if let Err(e) = open_in_file_manager(parent_dir) {
            eprintln!("Failed to open directory: {}", e);
        }
    } else {
        eprintln!("No parent directory found for file: {}", file_path);
    }
}

/// 跨平台打开文件所在目录
fn open_in_file_manager(path: &Path) -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("explorer").arg(path).spawn()?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(path).spawn()?;
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    } else {
        eprintln!("Unsupported platform for opening file location");
    }
    Ok(())
}
