//! 文件搜索工具 - 主程序入口
//!
//! 使用现代MVC架构组织的文件搜索应用程序

use anyhow::Context;
use netdisk_db::controllers::handlers::{
    handle_file_context_menu, handle_open_file, handle_open_file_location, send_to_aria2,
};
use netdisk_db::prelude::*; // 使用库的prelude简化导入
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, span, Level};
use tracing_subscriber;

/// 初始化应用程序配置
///
/// 如果配置文件不存在则创建默认配置
/// 并扫描当前目录下的数据库文件
fn initialize_config() -> Result<AppConfig> {
    let config_path = "config.json";

    let mut config = if std::path::Path::new(config_path).exists() {
        AppConfig::load_from_file(config_path).context("Failed to load config file")?
    } else {
        info!("Config file not found, creating default config");
        let default_config = AppConfig::default();
        default_config
            .save_to_file(config_path)
            .context("Failed to create default config file")?;
        default_config
    };

    // 扫描当前目录下的数据库文件
    scan_for_database_files(&mut config)?;

    // 记录配置信息
    debug!("Using database type: {}", config.database.db_type);
    debug!("Database connection: {}", config.database.connection_string);
    debug!("Current timestamp: {}", get_timestamp());
    debug!("File size formatting test: {}", format_file_size(1048576));
    debug!(
        "Available databases: {}",
        config.multi_database.databases.len()
    );

    Ok(config)
}

/// 扫描当前目录下的数据库文件
fn scan_for_database_files(config: &mut AppConfig) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    info!("Scanning for database files in: {:?}", current_dir);

    let mut found_databases = Vec::new();

    // 读取当前目录下的所有文件
    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // 检查是否为.db文件
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("db") {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        // 跳过临时文件和系统文件
                        if !file_name.starts_with('.') && !file_name.starts_with('~') {
                            let db_name = file_name.trim_end_matches(".db").to_string();
                            let db_path = path.to_string_lossy().to_string();

                            info!("Found database file: {} at path: {}", db_name, db_path);

                            found_databases.push(DatabaseConfig {
                                db_type: "sqlite".to_string(),
                                connection_string: db_path,
                                name: db_name,
                                description: Some(format!(
                                    "Auto-discovered database: {}",
                                    file_name
                                )),
                            });
                        }
                    }
                }
            }
        }
    }

    // 如果找到了数据库文件，更新配置
    if !found_databases.is_empty() {
        info!(
            "Found {} database files, updating configuration",
            found_databases.len()
        );

        // 清空现有的多数据库配置
        config.multi_database.databases.clear();

        // 添加发现的数据库
        for db_config in found_databases {
            config.add_database(db_config);
        }

        // 设置第一个发现的数据库为默认
        if !config.multi_database.databases.is_empty() {
            config.multi_database.default_database = 0;
            config.database = config.multi_database.databases[0].clone();
        }
    } else {
        info!("No database files found in current directory, using existing configuration");
    }

    Ok(())
}

/// 创建UI界面
///
/// # Arguments
/// * `config` - 应用配置（用于窗口大小等设置）
fn create_ui(config: &AppConfig) -> Result<AppWindow> {
    let ui = AppWindow::new().context("Failed to create UI window")?;

    // 可以在这里根据配置设置UI属性
    debug!(
        "UI window created with size: {}x{}",
        config.window_width, config.window_height
    );

    Ok(ui)
}

/// 设置事件处理器
///
/// # Arguments
/// * `ui` - UI 实例
/// * `database_manager` - 数据库管理器
fn setup_event_handlers(
    ui: &AppWindow,
    database_manager: Arc<Mutex<DatabaseManager>>,
) -> Result<()> {
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

    // 文件右键菜单处理
    let ui_handle = ui.as_weak();
    ui.on_file_context_menu_requested(move |file_item, x, y| {
        handle_file_context_menu(file_item, x, y, &ui_handle);
    });

    // 打开文件处理
    ui.on_open_file(move |file_path| {
        handle_open_file(&file_path);
    });

    // 打开文件位置处理
    ui.on_open_file_location(move |file_path| {
        handle_open_file_location(&file_path);
    });

    ui.on_send_to_aria2(move |file_path| {
        send_to_aria2(&file_path);
    });

    Ok(())
}

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
