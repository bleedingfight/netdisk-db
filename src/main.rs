mod config;
mod database;
mod ui;

use anyhow::{Context, Result};
use database::{sqlite::SqliteDatabase, Database};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::ui::AppWindow;

use tracing::{debug, error, info, span, Level};
use tracing_subscriber;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 创建一个 `span`，表示一个特定的执行范围
    let span = span!(Level::INFO, "netdisk_db", foo = 42, bar = "hello");

    // 使用 `span.enter()` 来指定该范围内的事件
    // `enter` 使得当前代码块的执行与这个 `span` 相关联
    let _enter = span.enter();

    // Initialize configuration
    let config_path = "config.json";
    let config = if std::path::Path::new(config_path).exists() {
        config::AppConfig::load_from_file(config_path).context("Failed to load config file")?
    } else {
        info!("Config file not found, creating default config");
        let default_config = config::AppConfig::default();
        default_config
            .save_to_file(config_path)
            .context("Failed to create default config file")?;
        default_config
    };

    debug!("Using database type: {}", config.database.db_type);
    debug!("Database connection: {}", config.database.connection_string);

    // Initialize database
    let database: Arc<Mutex<dyn Database>> = match config.database.db_type.as_str() {
        "sqlite" => {
            let sqlite_db = SqliteDatabase::new(&config.database.connection_string)
                .context("Failed to create SQLite database")?;
            sqlite_db
                .init_database()
                .context("Failed to initialize database")?;
            Arc::new(Mutex::new(sqlite_db))
        }
        _ => {
            anyhow::bail!("Unsupported database type: {}", config.database.db_type);
        }
    };

    // 创建 UI
    let ui = AppWindow::new().context("Failed to create UI window")?;

    let ui_handle = ui.as_weak();
    let database_handle = database.clone();
    let last_search_time = Arc::new(Mutex::new(Instant::now()));
    let search_delay = Duration::from_millis(300); // 300ms 防抖延迟

    // Handle search requests with debouncing
    ui.on_search_requested(move |query| {
        let ui = ui_handle.unwrap();
        let database = database_handle.clone();
        let query = query.to_string();
        let last_search_time = last_search_time.clone();
        let search_delay = search_delay;

        // 获取当前时间并检查防抖
        let now = Instant::now();
        let mut last_time = last_search_time.lock().unwrap();

        // 检查是否过了防抖时间
        if now.duration_since(*last_time) < search_delay {
            // 如果太快，直接返回，不执行搜索
            return;
        }

        // 更新最后搜索时间
        *last_time = now;
        drop(last_time); // 释放锁

        // 如果查询为空，清空结果
        if query.trim().is_empty() {
            let file_items = ModelRc::new(VecModel::default());
            ui.set_file_items(file_items);
            return;
        }

        // 执行搜索
        let results = database.lock().unwrap().search_files(&query);
        match results {
            Ok(results) => {
                let file_items = ui::file_records_to_model(results);
                ui.set_file_items(file_items);
            }
            Err(e) => {
                error!("Search failed: {}", e);
                // Clear results
                ui.set_file_items(ModelRc::new(VecModel::default()));
            }
        }
    });

    // Handle context menu requests
    let ui_handle = ui.as_weak();
    let database_handle = database.clone();
    ui.on_context_menu_requested(move |file_id, x, y| {
        let ui = ui_handle.unwrap();
        let database = database_handle.clone();

        debug!(
            "Context menu requested for file ID: {} at position ({}, {})",
            file_id, x, y
        );

        // 获取当前选中的文件信息
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();
        let file_name = selected_item.name.to_string();

        debug!("Selected file: {} at path: {}", file_name, file_path);

        // 这里可以添加更多的上下文菜单逻辑
        // 例如，根据文件类型启用/禁用某些菜单项
    });

    // Handle context menu actions
    let ui_handle = ui.as_weak();
    let database_handle = database.clone();

    // 下载文件功能
    ui.on_download_file(move || {
        debug!("=== DOWNLOAD FILE FUNCTION CALLED ===");

        let ui = ui_handle.unwrap();
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();
        let file_name = selected_item.name.to_string();

        debug!(
            "Attempting to download file: {} from path: {}",
            file_name, file_path
        );

        // 检查文件是否存在
        let path = Path::new(&file_path);
        if path.exists() {
            debug!("✓ File found: {}", file_path);
            debug!(
                "✓ File size: {} bytes",
                std::fs::metadata(path).unwrap().len()
            );
            debug!("✓ File type: {:?}", selected_item.file_type);

            // 模拟下载过程
            debug!("→ Starting download simulation...");
            debug!("→ Copying file to downloads directory...");
            debug!("→ Download completed successfully!");

            // 这里可以添加实际的文件复制逻辑
            let downloads_dir = std::path::PathBuf::from(".");
            let target_path = downloads_dir.join(&file_name);
            debug!("→ File would be saved to: {:?}", target_path);
        } else {
            error!("✗ File not found: {}", file_path);
            error!("✗ Download failed - file does not exist");
        }

        debug!("=== DOWNLOAD FILE FUNCTION COMPLETED ===");
    });

    // 发送到服务器功能
    let ui_handle = ui.as_weak();
    ui.on_send_to_server(move || {
        let ui = ui_handle.unwrap();
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();
        let file_name = selected_item.name.to_string();

        debug!(
            "Sending file to server: {} from path: {}",
            file_name, file_path
        );

        // 在实际应用中，这里可以实现发送到服务器的逻辑
        // 例如，通过HTTP POST请求上传文件
        if Path::new(&file_path).exists() {
            debug!("File exists and can be sent to server");
            // 这里可以添加实际的上传逻辑
        } else {
            error!("File not found: {}", file_path);
        }
    });

    // 更新文件内容功能
    let ui_handle = ui.as_weak();
    ui.on_update_content(move || {
        let ui = ui_handle.unwrap();
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();
        let file_name = selected_item.name.to_string();

        debug!(
            "Updating file content: {} at path: {}",
            file_name, file_path
        );

        // 在实际应用中，这里可以实现文件内容更新逻辑
        // 例如，打开文件编辑器或读取文件内容进行修改
        if Path::new(&file_path).exists() {
            debug!("File exists and can be updated");
            // 这里可以添加实际的文件更新逻辑
        } else {
            error!("File not found: {}", file_path);
        }
    });

    // 打开文件位置功能
    let ui_handle = ui.as_weak();
    ui.on_open_location(move || {
        let ui = ui_handle.unwrap();
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();

        debug!("Opening file location: {}", file_path);

        // 在实际应用中，这里可以实现打开文件位置的功能
        // 例如，在文件管理器中打开文件所在目录
        if let Some(parent_dir) = Path::new(&file_path).parent() {
            debug!("Opening directory: {:?}", parent_dir);
            // 这里可以添加实际的打开目录逻辑
        }
    });

    // Run application
    ui.run().context("Failed to run UI application")?;

    Ok(())
}
