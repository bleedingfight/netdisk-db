//! 文件搜索工具 - 主程序入口
//!
//! 使用现代MVC架构组织的文件搜索应用程序

use anyhow::Context;
use netdisk_db::prelude::*; // 使用库的prelude简化导入
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, span, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt::init();

    // 创建应用范围跟踪
    let span = span!(Level::INFO, "netdisk_db", foo = 42, bar = "hello");
    let _enter = span.enter();

    info!("Starting File Search Application");

    // 初始化配置
    let config = initialize_config()?;
    debug!("Configuration loaded successfully");

    // 初始化数据库管理器
    let config_arc = Arc::new(Mutex::new(config.clone()));
    let database_manager = Arc::new(Mutex::new(DatabaseManager::new(config_arc.clone())?));
    debug!("Database manager initialized successfully");

    // 创建UI
    let ui = create_ui(&config)?;
    debug!("UI created successfully");

    // 设置事件处理器
    setup_event_handlers(&ui, database_manager.clone())?;
    
    // 初始化数据库选择器
    initialize_database_selector(&ui.as_weak(), database_manager.clone());

    info!("Application initialized, starting main loop");
    
    // 运行应用
    ui.run().context("Failed to run UI application")?;
    
    info!("Application shutdown");
    Ok(())
}

/// 初始化应用程序配置
/// 
/// 如果配置文件不存在则创建默认配置
fn initialize_config() -> Result<AppConfig> {
    let config_path = "config.json";
    
    let config = if std::path::Path::new(config_path).exists() {
        AppConfig::load_from_file(config_path).context("Failed to load config file")?
    } else {
        info!("Config file not found, creating default config");
        let default_config = AppConfig::default();
        default_config.save_to_file(config_path)
            .context("Failed to create default config file")?;
        default_config
    };

    // 记录配置信息
    debug!("Using database type: {}", config.database.db_type);
    debug!("Database connection: {}", config.database.connection_string);
    debug!("Current timestamp: {}", get_timestamp());
    debug!("File size formatting test: {}", format_file_size(1048576));

    Ok(config)
}

/// 初始化数据库连接
/// 
/// # Arguments
/// * `config` - 应用配置
async fn initialize_database(config: &AppConfig) -> Result<Arc<Mutex<dyn Database>>> {
    match config.database.db_type.as_str() {
        "sqlite" => {
            let sqlite_db = SqliteDatabase::new(&config.database.connection_string)
                .context("Failed to create SQLite database")?;
            sqlite_db.init_database()
                .context("Failed to initialize database")?;
            Ok(Arc::new(Mutex::new(sqlite_db)))
        }
        _ => {
            anyhow::bail!("Unsupported database type: {}", config.database.db_type);
        }
    }
}

/// 创建UI界面
/// 
/// # Arguments
/// * `config` - 应用配置（用于窗口大小等设置）
fn create_ui(config: &AppConfig) -> Result<AppWindow> {
    let ui = AppWindow::new().context("Failed to create UI window")?;
    
    // 可以在这里根据配置设置UI属性
    debug!("UI window created with size: {}x{}", 
           config.window_width, config.window_height);
    
    Ok(ui)
}

/// 设置事件处理器
///
/// # Arguments
/// * `ui` - UI 实例
/// * `database_manager` - 数据库管理器
fn setup_event_handlers(ui: &AppWindow, database_manager: Arc<Mutex<DatabaseManager>>) -> Result<()> {
    let ui_handle = ui.as_weak();
    let database_handle = database_manager.lock().unwrap().get_current_database();
    let last_search_time = Arc::new(Mutex::new(Instant::now()));
    let search_delay = Duration::from_millis(300); // 300ms 防抖延迟

    // 搜索请求处理
    ui.on_search_requested(move |query| {
        handle_search_request(
            &query,
            &ui_handle.clone(),
            database_handle.clone(),
            last_search_time.clone(),
            search_delay,
        );
    });

    // 数据库切换处理
    let ui_handle = ui.as_weak();
    let manager_handle = database_manager.clone();
    ui.on_database_changed(move |index| {
        handle_database_changed(index, &ui_handle, manager_handle.clone());
    });

    // 上下文菜单请求处理
    let ui_handle = ui.as_weak();
    let database_handle = database_manager.lock().unwrap().get_current_database();
    ui.on_context_menu_requested(move |file_id, x, y| {
        handle_context_menu_requested(
            file_id,
            x,
            y,
            &ui_handle,
            database_handle.clone(),
        );
    });

    // 文件操作处理
    setup_file_operations(ui, database_manager)?;

    Ok(())
}

/// 设置文件操作处理器
///
/// # Arguments
/// * `ui` - UI 实例
/// * `database_manager` - 数据库管理器
fn setup_file_operations(ui: &AppWindow, _database_manager: Arc<Mutex<DatabaseManager>>) -> Result<()> {
    let ui_handle = ui.as_weak();

    // 下载文件
    ui.on_download_file(move || {
        let ui_handle = ui_handle.clone();
        let ui = match ui_handle.upgrade() {
            Some(u) => u,
            None => return,
        };

        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();

        debug!("Starting download for file: {}", file_path);

        // 启动后台任务
        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                download_proc(file_path.clone());
            })
            .await
            .unwrap();

            // 回到 UI 线程更新
            if let Some(ui) = ui_handle.upgrade() {
                ui.invoke_update_content();
            }
        });
    });

    // 发送到服务器
    let ui_handle = ui.as_weak();
    ui.on_send_to_server(move || {
        handle_send_to_server(&ui_handle);
    });

    // 更新内容
    let ui_handle = ui.as_weak();
    ui.on_update_content(move || {
        handle_update_content(&ui_handle);
    });
    
    // 打开文件位置
    let ui_handle = ui.as_weak();
    ui.on_open_location(move || {
        handle_open_location(&ui_handle);
    });

    // 发送到 Aria2
    let ui_handle = ui.as_weak();
    ui.on_send_to_aria2(move || {
        let ui_handle = ui_handle.clone();
        let ui = match ui_handle.upgrade() {
            Some(u) => u,
            None => return,
        };

        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();

        debug!("Sending file to Aria2: {}", file_path);

        // 启动后台任务
        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                send_to_aria2(file_path.clone());
            })
            .await
            .unwrap();

            // 回到 UI 线程更新
            if let Some(ui) = ui_handle.upgrade() {
                ui.invoke_update_content();
            }
        });
    });

    Ok(())
}
