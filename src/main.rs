//! 文件搜索工具 - 主程序入口
//!
//! 使用现代MVC架构组织的文件搜索应用程序

use actix_web::{web, HttpServer};
use anyhow::Context;
use arboard::Clipboard;
use netdisk_core::create_app;
use netdisk_core::netdisk_api::prelude::get_access_token_from_cache;
use netdisk_core::netdisk_auth::basic_env::NetDiskEnv;
use netdisk_core::responses::prelude::AccessToken;
use netdisk_db::controllers::handlers::copy_to_clipboard;
use netdisk_db::controllers::handlers::{
    get_file_url, handle_file_context_menu, handle_open_file, handle_open_file_location, send_to_aria2,
};
use netdisk_db::prelude::*; // 使用库的prelude简化导入
use netdisk_db::services::aria2::{create_shared_aria2_service, SharedAria2Service};
use slint::ComponentHandle;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::task;
use tracing::{debug, error, info, span, warn, Level};
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
/// * `aria2_service` - Aria2服务实例
fn setup_event_handlers(
    ui: &AppWindow,
    database_manager: Arc<Mutex<DatabaseManager>>,
    aria2_service: SharedAria2Service,
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

    // ui.on_search_requested(move |query| {
    //     // 1. 克隆所有需要在异步任务中使用的句柄和值
    //     let ui_handle = ui_handle.clone();
    //     let database_handle = database_handle.clone();
    //     let last_search_time = last_search_time.clone();

    //     // search_delay 是基本类型 (如 u64)，可以直接复制
    //     // query 是 String，已被 move 进回调闭包，现在 move 进 task

    //     // 2. 使用 tokio::task::spawn_blocking 将耗时的同步操作移到阻塞线程池
    //     task::spawn_blocking(move || {
    //         // 这个闭包在专用的阻塞线程上运行，不会阻塞 UI 线程

    //         // 3. 在新线程中执行同步搜索逻辑
    //         handle_search_request(
    //             &query, // 现在 query 是该线程拥有的 String
    //             &ui_handle,
    //             database_handle,
    //             last_search_time,
    //             search_delay,
    //         );

    //         // 注意：如果 handle_search_request 返回结果，您可能需要将结果返回
    //         // 并在主异步线程中使用 .await 接收它（如果需要）
    //     });

    //     // UI 回调立即返回，保持 UI 响应性
    // });
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

    ui.on_send_to_aria2({
        let ui_weak = ui.as_weak();
        let aria2_service_clone = aria2_service.clone();
        move |file_path, etag, size_kb| {
            let ui_handle = ui_weak.clone();
            let aria2_service_inner = aria2_service_clone.clone();
            let path = file_path.to_string();
            let tag = etag.to_string();
            let size_bytes = size_kb.to_string().trim().parse::<u64>().unwrap();
            debug!(
                "Sending to Aria2: path={}, etag={}, size_bytes={}",
                path, tag, size_bytes
            );

            let _ = slint::spawn_local(async move {
                // 首先尝试使用本地Aria2服务
                if let Some(aria2_client) = aria2_service_inner.lock().unwrap().get_client() {
                    match get_file_url(&path, &tag, size_bytes).await {
                        Ok(download_url) => {
                            match aria2_client.add_download(&download_url, None).await {
                                Ok(gid) => {
                                    info!("Download task added to Aria2 with GID: {}", gid);
                                    if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_search_text("下载任务已添加到Aria2".into());
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to add download to Aria2: {}", e);
                                    if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_search_text(format!("Aria2添加失败: {}", e).into());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to get download URL: {}", e);
                            if let Some(ui) = ui_handle.upgrade() {
                                ui.set_search_text(format!("获取下载链接失败: {}", e).into());
                            }
                        }
                    }
                } else {
                    // 回退到原来的HTTP方式
                    warn!("Aria2 client not available, falling back to HTTP method");
                    match send_to_aria2(path, tag, size_bytes).await {
                        Ok(_) => {
                            if let Some(ui) = ui_handle.upgrade() {
                                ui.set_search_text("上传成功".into());
                            }
                        }
                        Err(e) => {
                            if let Some(ui) = ui_handle.upgrade() {
                                ui.set_search_text(format!("请求失败: {}", e).into());
                            }
                        }
                    }
                }
            });
        }
    });
    let clipboard = Arc::new(Mutex::new(Clipboard::new()?));
    ui.on_copy_to_clipboard({
        let ui_weak = ui.as_weak();
        let clipboard_ref = Arc::clone(&clipboard);
        move |file_path, etag, size_kb| {
            let ui_handle = ui_weak.clone();
            let clipboard_inner = Arc::clone(&clipboard_ref);
            let path = file_path.to_string();
            let tag = etag.to_string();
            let size_bytes = size_kb.to_string().trim().parse::<u64>().unwrap();
            let _ = slint::spawn_local(async move {
                let mut clipboard = clipboard_inner.lock().unwrap();
                match copy_to_clipboard(path, tag, size_bytes, &mut *clipboard).await {
                    Ok(_) => {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_search_text("成功获取链接".into());
                        }
                    }
                    Err(e) => {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_search_text(format!("无法获取链接: {}", e).into());
                        }
                    }
                }
            });

        }
    });

    Ok(())
}

pub async fn start_backend_service(port: u16) -> io::Result<()> {
    // 1. 初始化配置和环境
    let env = match NetDiskEnv::new() {
        Ok(env) => env,
        Err(e) => {
            error!("❌ 致命错误：无法初始化 NetDiskEnv：{}", e);
            // 无法初始化环境，直接返回 IO 错误
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to initialize NetDiskEnv: {}", e),
            ));
        }
    };

    // 2. 尝试加载 AccessToken
    let file_path = env.config_dir.join("config.toml");
    let mut access_token: AccessToken = AccessToken::default();
    match get_access_token_from_cache(&file_path).await {
        Ok(token) => {
            access_token = token;
        }
        Err(_) => {
            error!("Error to message: Failed to load access token from cache.");
        }
    }

    // 3. 注入全局数据 (web::Data 是 Arc 的封装，用于线程间共享)
    // 推荐在外部先创建 Arc，再创建 web::Data
    let config_path_data = web::Data::new(env);
    let access_token_data = web::Data::new(access_token);

    let addr = format!("127.0.0.1:{}", port);
    info!("Web 后端服务正在绑定到：{}", addr);

    // 4. 启动 Actix Web 服务器
    // 注意：HttpServer::new 接收一个 move 闭包
    let server = HttpServer::new(move || {
        // 在每次新 worker 线程创建时，克隆 web::Data
        create_app(config_path_data.clone(), access_token_data.clone())
    })
    .bind(addr)?; // 绑定端口，如果失败会返回 io::Error

    // 运行服务器并等待
    server.run().await
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt::init();

    // 创建应用范围跟踪
    let span = span!(Level::INFO, "netdisk_db", foo = 42, bar = "hello");
    let _enter = span.enter();

    info!("Starting File Search Application");
    let port = 8080;

    // 初始化配置
    let config = initialize_config()?;
    debug!("Configuration loaded successfully");

    // 启动Aria2服务
    let aria2_service = create_shared_aria2_service(config.aria2.clone());
    {
        let mut aria2_service_lock = aria2_service.lock().unwrap();
        if let Err(e) = aria2_service_lock.start() {
            error!("Failed to start Aria2 service: {}", e);
        } else {
            // 等待Aria2服务就绪
            let aria2_ready = aria2_service_lock.wait_until_ready(10).await;
            if aria2_ready {
                info!("Aria2 service is ready");
            } else {
                warn!("Aria2 service is not ready, download functionality may not work");
            }
        }
    }

    // 启动后端服务 - 使用 spawn_blocking 因为 HttpServer 不是 Send
    let _server_handle = task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { start_backend_service(port).await })
    });

    // 初始化数据库管理器
    let config_arc = Arc::new(Mutex::new(config.clone()));
    let database_manager = Arc::new(Mutex::new(DatabaseManager::new(config_arc.clone())?));
    debug!("Database manager initialized successfully");

    // 创建UI
    let ui = create_ui(&config)?;
    debug!("UI created successfully");

    // 设置事件处理器（传递aria2服务）
    setup_event_handlers(&ui, database_manager.clone(), aria2_service.clone())?;

    // 初始化数据库选择器
    initialize_database_selector(&ui.as_weak(), database_manager.clone());

    info!("Application initialized, starting main loop");

    // 运行应用
    ui.run().context("Failed to run UI application")?;

    info!("Application shutdown");
    Ok(())
}
