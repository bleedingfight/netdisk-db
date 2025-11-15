//! 控制器处理模块 - 业务逻辑和事件处理
//!
//! 包含所有用户交互和业务流程的处理函数

use crate::models::database::Database;
use crate::services::database_manager::DatabaseManager;
use crate::views::ui::{database_list_to_string_model, file_records_to_model, AppWindow, FileItem};
use actix_web::Result;
use arboard::Clipboard;
use netdisk_core::responses::prelude::{DownloadUrlResponse, FileQuery, UploadFileResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")] // 关键！将 Rust 的 snake_case 映射到 JSON 的 camelCase
pub struct UploadFileItemPayload {
    #[serde(alias = "parentFileID")]
    pub parent_file_id: i64,
    pub filename: String, // 字段类型必须确定，不能是泛型 T
    pub etag: String,
    pub size: u64,
}

/// 发送文件上传请求到服务器
///
/// # Arguments
/// * `data` - 文件上传数据
/// # Returns
/// * `Result<()>` - 成功返回 Ok，失败返回错误
pub async fn send_file_upload_request(
    client: &Client,
    data: UploadFileItemPayload,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = "http://127.0.0.1:8080/file/upload";

    info!("正在发送文件上传 POST 请求到: {}", url);
    debug!("请求数据: {:?}", data);

    // 发送 POST 请求
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&data)
        .send()
        .await?;

    let status = response.status();
    info!("响应状态: {}", status);

    // 检查 HTTP 状态码是否为成功状态
    if !status.is_success() {
        let error_text = response.text().await?;
        return Err(format!(
            "HTTP 请求失败，状态码: {}，错误信息: {}",
            status, error_text
        )
        .into());
    } else {
        // info!("文件上传请求成功，状态码: {:?}", &response.text().await?);
        let resp: UploadFileResponse = serde_json::from_str(&response.text().await?)?;
        let url = resp
            .data
            .ok_or("响应数据确实，也许数据已经上传过了....")?
            .file_id
            .ok_or("服务器列表为空")?
            .to_string();
        Ok(url)
    }
}

/// TODO 获取函数的url这个函数有问题
pub async fn get_download_url(
    client: &Client,
    query: &FileQuery,
) -> Result<DownloadUrlResponse, Box<dyn std::error::Error>> {
    // 基础请求 URL
    let base_url = "http://127.0.0.1:8080/file/download";

    // 发送 GET 请求，携带查询参数
    let response = client
        .get(base_url)
        .query(query) // 自动将 FileQuery 转为 URL 查询参数（如 ?fileId=19349166）
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| Box::new(e))?;

    // 检查 HTTP 状态码
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("请求失败，状态码: {}, 错误信息: {}", status, error_body),
        )));
    }

    // 将响应体反序列化为 DownloadUrlResponse
    let download_response: DownloadUrlResponse = response.json().await.map_err(|e| Box::new(e))?;

    // 检查业务状态码（如果接口用 code 字段表示业务成功）
    if download_response.code != 0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "业务处理失败: code={}, message={}",
                download_response.code, download_response.message
            ),
        )));
    }

    Ok(download_response)
}

/// 规范化文件名称，这里需要重构
pub fn format_upload_filename<T>(filename: T) -> Option<String>
where
    T: AsRef<Path>,
{
    let path = filename.as_ref();

    path.file_name()
        // 3. 将 &OsStr 转换为 String (或 &str)
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_string())
}
/// 发送到 Aria2 处理函数（模拟实现）
///
/// # Arguments
/// * `_url` - 文件URL或路径
pub async fn send_to_aria2<T>(path: T, etag: T, size: u64) -> Result<(), Box<dyn std::error::Error>>
where
    T: AsRef<str> + std::fmt::Debug,
{
    debug!(
        "Send {:?} {:?} [{}] to Aria2 proc started",
        &path, &etag, size
    );

    // 构造请求体数据
    let name = format_upload_filename(path.as_ref()).unwrap();
    let payload = UploadFileItemPayload {
        parent_file_id: 0,
        filename: name,
        etag: etag.as_ref().to_string(),
        size: size,
    };

    // 创建 HTTP 客户端
    let client = Client::new();
    // 发送文件上传请求
    let mut file_id: String = String::new();
    match send_file_upload_request(&client, payload).await {
        Ok(mesg) => {
            file_id = mesg.clone();
            info!("后台服务请求成功完成。{:?}", &mesg);
        }
        Err(e) => {
            error!("请求失败，错误信息: {}", e);
            return Err(e);
        }
    }
    let query = FileQuery {
        file_id: file_id.parse::<i64>().unwrap_or(0),
    };
    debug!("准备获取下载链接，查询参数: {:?}", &query);

    let mut link = String::new();
    match get_download_url(&client, &query).await {
        Ok(download_response) => {
            if let Some(data) = download_response.data {
                info!("响应数据: {:?}", &data.download_url);
                link = data.download_url;
                // if let Some(urls) = data.download_url {
                //     if let Some(first_url) = urls.first() {
                //         info!("获取到下载链接: {}", first_url);
                //         // 这里可以调用实际的 Aria2 接口来添加下载任务
                //         // 例如：aria2.addUri([first_url], options);
                //     } else {
                //         error!("下载链接列表为空");
                //     }
                // } else {
                //     error!("响应数据中没有下载链接");
                // }
            } else {
                error!("响应数据为空");
            }
        }
        Err(_e) => {
            error!("获取下载链接失败，错误信息: {}", _e);
            return Err(_e);
        }
    }

    debug!("Send to Aria2 proc finished");
    Ok(())
}
pub async fn get_file_url<T>(
    path: T,
    etag: T,
    size: u64,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: AsRef<str> + std::fmt::Debug,
{
    let name = format_upload_filename(path.as_ref()).unwrap();
    let payload = UploadFileItemPayload {
        parent_file_id: 0,
        filename: name,
        etag: etag.as_ref().to_string(),
        size: size,
    };

    let client = Client::new();
    // 发送文件上传请求
    let mut file_id: String = String::new();
    match send_file_upload_request(&client, payload).await {
        Ok(mesg) => {
            file_id = mesg.clone();
            info!("后台服务请求成功完成。{:?}", &mesg);
        }
        Err(e) => {
            error!("请求失败，错误信息: {}", e);
            return Err(e);
        }
    }
    let query = FileQuery {
        file_id: file_id.parse::<i64>().unwrap_or(0),
    };
    debug!("准备获取下载链接，查询参数: {:?}", &query);

    let mut link = String::new();
    match get_download_url(&client, &query).await {
        Ok(download_response) => {
            if let Some(data) = download_response.data {
                info!("响应数据: {:?}", &data.download_url);
                link = data.download_url;
            } else {
                error!("响应数据为空");
            }
        }
        Err(_e) => {
            error!("获取下载链接失败，错误信息: {}", _e);
            return Err(_e);
        }
    }
    Some(link).ok_or("无法获取下载链接".into())
}

/// 发送到 url 到系统剪切板
///
/// # Arguments
/// * `path` - 文件路径
/// * `etag` - 文件ETag
/// * `size` - 文件大小
/// * `clipboard` - 持久化的剪切板实例引用
pub async fn copy_to_clipboard<T>(
    path: T,
    etag: T,
    size: u64,
    clipboard: &mut Clipboard,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: AsRef<str> + std::fmt::Debug,
{
    let link = get_file_url(path, etag, size).await?;

    debug!("==>Copying link to clipboard: {}", &link);

    // 尝试复制到剪切板，最多重试3次
    let mut attempts = 0;
    let max_attempts = 3;

    while attempts < max_attempts {
        match clipboard.set_text(&link) {
            Ok(_) => {
                info!("成功复制链接到剪切板: {}", &link);

                // 保持剪切板实例存活，避免过早丢弃
                // 短暂延迟确保剪切板管理器有足够时间读取内容
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                return Ok(link);
            }
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    error!("复制到剪切板失败，已重试{}次: {}", attempts, e);
                    return Err(Box::new(e));
                }
                debug!("复制到剪切板失败，第{}次重试: {}", attempts, e);
                // 等待一段时间后重试
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    Err("无法复制到剪切板".into())
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
    debug!("尝试执行搜索任务");
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
pub fn handle_file_context_menu(file_item: FileItem, x: f32, y: f32, ui: &slint::Weak<AppWindow>) {
    info!("=== RIGHT CLICK DETECTED ===");
    info!("File: {}, Position: ({}, {})", file_item.name, x, y);

    let ui = match ui.upgrade() {
        Some(u) => u,
        None => {
            error!("Failed to upgrade UI handle");
            return;
        }
    };

    info!(
        "Context menu requested for file: {} at position ({}, {})",
        file_item.name, x, y
    );

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
        let _ = std::process::Command::new("open").arg(file_path).spawn();
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

    debug!(
        "Initialized database selector with {} databases",
        manager.get_database_list().len()
    );
}
