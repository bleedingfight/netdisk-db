mod config;
mod database;
mod handlers;
mod ui;

use anyhow::{Context, Result};
use database::sqlite::SqliteDatabase;
use database::Database;
use ui::AppWindow;
use slint::ComponentHandle;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use handlers::*;
use tracing::{debug, info, span, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
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

    ui.on_search_requested(move |query| {
        handle_search_request(
            &query,
            &ui_handle.clone(),
            database_handle.clone(),
            last_search_time.clone(),
            search_delay,
        );
    });

    // Handle context menu requests
    let ui_handle = ui.as_weak();
    let database_handle = database.clone();
    ui.on_context_menu_requested(move |file_id, x, y| {
        handle_context_menu_requested(
            file_id,
            x,
            y,
            &ui_handle, // UI 句柄
            database_handle.clone(),
        );
    });

    // Handle context menu actions
    let ui_handle = ui.as_weak();
    let _database_handle = database.clone();

    // 下载文件功能
    ui.on_download_file(move || {
        // 获取 UI 弱引用，防止闭包持有 UI 的强引用导致内存泄漏
        let ui_handle = ui_handle.clone();
        let ui = ui_handle.unwrap();

        // 复制选中项，避免闭包捕获 UI 的强引用
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();
        let _file_name = selected_item.name.to_string();

        // 启动后台任务
        tokio::spawn(async move {
            // 如果 download_proc 是同步阻塞函数，用 spawn_blocking
            tokio::task::spawn_blocking(move || {
                handlers::download_proc(file_path.clone());
            })
            .await
            .unwrap(); // 等待后台线程完成

            // 回到 UI 线程更新
            if let Some(ui) = ui_handle.upgrade() {
                ui.invoke_update_content();
            }
        });
    });

    // 发送到服务器功能
    let ui_handle = ui.as_weak();
    ui.on_send_to_server(move || {
        handle_send_to_server(&ui_handle);
    });

    // 更新文件内容功能
    let ui_handle = ui.as_weak();
    let handle = ui_handle.clone();
    ui.on_update_content(move || {
        handle_update_content(&handle);
    });
    
    // 打开文件位置功能
    let handle = ui_handle.clone();
    ui.on_open_location(move || {
        handle_open_location(&handle);
    });

    let handle = ui_handle.clone();
    ui.on_open_location(move || {
        handle_open_location(&handle);
    });
    ui.on_send_to_aria2(move || {
        let ui_handle = ui_handle.clone();
        let ui = ui_handle.unwrap();
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();

        debug!("Try to send {} to aria2 server!", file_path);

        // 启动后台任务
        tokio::spawn(async move {
            // 如果 download_proc 是同步阻塞函数，用 spawn_blocking
            tokio::task::spawn_blocking(move || {
                handlers::send_to_aria2(file_path.clone());
            })
            .await
            .unwrap(); // 等待后台线程完成

            // 回到 UI 线程更新
            if let Some(ui) = ui_handle.upgrade() {
                ui.invoke_update_content();
            }
        });
    });

    // Run application
    ui.run().context("Failed to run UI application")?;

    Ok(())
}
