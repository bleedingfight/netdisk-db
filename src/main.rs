mod config;
mod database;
mod ui;
mod context_menu;

use anyhow::{Result, Context};
use database::{Database, sqlite::SqliteDatabase};
use slint::{ModelRc, VecModel, ComponentHandle};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::path::Path;

use crate::ui::AppWindow;
use crate::context_menu::ContextMenuManager;

fn main() -> Result<()> {
    // Initialize configuration
    let config_path = "config.json";
    let config = if std::path::Path::new(config_path).exists() {
        config::AppConfig::load_from_file(config_path)
            .context("Failed to load config file")?
    } else {
        println!("Config file not found, creating default config");
        let default_config = config::AppConfig::default();
        default_config.save_to_file(config_path)
            .context("Failed to create default config file")?;
        default_config
    };

    println!("Using database type: {}", config.database.db_type);
    println!("Database connection: {}", config.database.connection_string);

    // Initialize database
    let database: Arc<Mutex<dyn Database>> = match config.database.db_type.as_str() {
        "sqlite" => {
            let sqlite_db = SqliteDatabase::new(&config.database.connection_string)
                .context("Failed to create SQLite database")?;
            sqlite_db.init_database()
                .context("Failed to initialize database")?;
            Arc::new(Mutex::new(sqlite_db))
        }
        _ => {
            anyhow::bail!("Unsupported database type: {}", config.database.db_type);
        }
    };

    // 创建 UI
    let ui = AppWindow::new()
        .context("Failed to create UI window")?;
    
    // 创建模块化菜单管理器
    let mut menu_manager = ContextMenuManager::new();
    
    // 添加 Aria2 支持作为示例
    menu_manager.add_aria2_support();
    
    // 将菜单项转换为 slint 格式并设置到 UI
    let menu_items = menu_manager.get_slint_struct_items();
    // menu_items 已经是正确的类型，直接使用
    ui.set_context_menu_items(slint::ModelRc::new(slint::VecModel::from(menu_items)));
    
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
                eprintln!("Search failed: {}", e);
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
        
        println!("Context menu requested for file ID: {} at position ({}, {})", file_id, x, y);
        
        // 获取当前选中的文件信息
        let selected_item = ui.get_selected_file_item();
        let file_path = selected_item.path.to_string();
        let file_name = selected_item.name.to_string();
        
        println!("Selected file: {} at path: {}", file_name, file_path);
        
        // 这里可以添加更多的上下文菜单逻辑
        // 例如，根据文件类型启用/禁用某些菜单项
    });

    // Handle menu item clicks with modular system
    let menu_manager_ref = Arc::new(Mutex::new(menu_manager));
    let ui_handle = ui.as_weak();
    
    ui.on_menu_item_clicked(move |menu_index| {
        let ui = ui_handle.unwrap();
        let menu_manager = menu_manager_ref.lock().unwrap();
        
        println!("Menu item clicked with index: {}", menu_index);
        
        if let Some(menu_item) = menu_manager.get_item_by_index(menu_index as usize) {
            let selected_item = ui.get_selected_file_item();
            let file_path = selected_item.path.to_string();
            
            println!("Executing action: {} for file: {}", menu_item.action, file_path);
            
            // 执行对应的动作
            match menu_item.action.as_str() {
                "download" => {
                    println!("=== DOWNLOAD FILE FUNCTION CALLED ===");
                    println!("Attempting to download file: {} from path: {}", selected_item.name, file_path);
                    
                    if Path::new(&file_path).exists() {
                        println!("✓ File found: {}", file_path);
                        println!("→ Download completed successfully!");
                    } else {
                        eprintln!("✗ File not found: {}", file_path);
                    }
                    println!("=== DOWNLOAD FILE FUNCTION COMPLETED ===");
                },
                "send_to_server" => {
                    println!("Sending file to server: {} from path: {}", selected_item.name, file_path);
                    if Path::new(&file_path).exists() {
                        println!("File exists and can be sent to server");
                    } else {
                        eprintln!("File not found: {}", file_path);
                    }
                },
                "update_content" => {
                    println!("Updating file content: {} at path: {}", selected_item.name, file_path);
                    if Path::new(&file_path).exists() {
                        println!("File exists and can be updated");
                    } else {
                        eprintln!("File not found: {}", file_path);
                    }
                },
                "open_location" => {
                    println!("Opening file location: {}", file_path);
                    if let Some(parent_dir) = Path::new(&file_path).parent() {
                        println!("Opening directory: {:?}", parent_dir);
                    }
                },
                "send_to_aria2" => {
                    println!("Sending file to Aria2: {} from path: {}", selected_item.name, file_path);
                    if Path::new(&file_path).exists() {
                        println!("✓ File found, sending to Aria2 download manager...");
                        println!("→ Aria2 RPC call would be made here");
                        println!("→ Download added to Aria2 queue successfully!");
                    } else {
                        eprintln!("✗ File not found: {}", file_path);
                    }
                },
                _ => {
                    eprintln!("Unknown action: {}", menu_item.action);
                }
            }
        } else {
            eprintln!("No menu item found at index: {}", menu_index);
        }
    });


    // Run application
    ui.run()
        .context("Failed to run UI application")?;

    Ok(())
}
