//! 数据库管理器 - 支持动态数据库切换
//!
//! 提供数据库实例的动态创建和管理功能

use anyhow::{Result, Context};
use std::sync::{Arc, Mutex};
use crate::models::config::{AppConfig, DatabaseConfig};
use crate::models::database::Database;
use crate::services::database::sqlite::SqliteDatabase;
use tracing::{debug, info};

/// 数据库管理器
pub struct DatabaseManager {
    current_database: Arc<Mutex<dyn Database>>,
    config: Arc<Mutex<AppConfig>>,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub fn new(config: Arc<Mutex<AppConfig>>) -> Result<Self> {
        let current_db = Self::create_database(&config.lock().unwrap().database)?;
        
        Ok(Self {
            current_database: current_db,
            config,
        })
    }
    
    /// 获取当前数据库实例
    pub fn get_current_database(&self) -> Arc<Mutex<dyn Database>> {
        self.current_database.clone()
    }
    
    /// 切换到指定数据库
    pub fn switch_database(&mut self, index: usize) -> Result<()> {
        let mut config = self.config.lock().unwrap();
        
        // 切换到新的数据库配置
        config.switch_database(index)?;
        
        // 创建新的数据库实例
        let new_db = Self::create_database(&config.database)?;
        
        // 更新当前数据库
        self.current_database = new_db;
        
        info!("Switched to database: {} (index: {})", 
              config.database.name, index);
        
        Ok(())
    }
    
    /// 获取当前数据库信息
    pub fn get_current_database_info(&self) -> (String, String) {
        let config = self.config.lock().unwrap();
        let db_config = &config.database;
        (db_config.name.clone(), db_config.db_type.clone())
    }
    
    /// 获取数据库列表
    pub fn get_database_list(&self) -> Vec<(String, String, usize)> {
        let config = self.config.lock().unwrap();
        config.multi_database.databases
            .iter()
            .enumerate()
            .map(|(i, db)| (db.name.clone(), db.db_type.clone(), i))
            .collect()
    }
    
    /// 获取当前数据库索引
    pub fn get_current_database_index(&self) -> usize {
        let config = self.config.lock().unwrap();
        config.current_database_index()
    }
    
    /// 根据配置创建数据库实例
    fn create_database(db_config: &DatabaseConfig) -> Result<Arc<Mutex<dyn Database>>> {
        debug!("Creating database instance: {} ({})", db_config.name, db_config.db_type);
        
        match db_config.db_type.as_str() {
            "sqlite" => {
                let sqlite_db = SqliteDatabase::new(&db_config.connection_string)
                    .context("Failed to create SQLite database")?;
                sqlite_db.init_database()
                    .context("Failed to initialize database")?;
                Ok(Arc::new(Mutex::new(sqlite_db)))
            }
            _ => {
                anyhow::bail!("Unsupported database type: {}", db_config.db_type);
            }
        }
    }
    
    /// 添加新数据库配置
    pub fn add_database(&mut self, config: DatabaseConfig) -> Result<()> {
        let mut app_config = self.config.lock().unwrap();
        app_config.add_database(config);
        Ok(())
    }
    
    /// 移除数据库配置
    pub fn remove_database(&mut self, index: usize) -> Result<()> {
        let mut app_config = self.config.lock().unwrap();
        app_config.remove_database(index)?;
        
        // 如果移除了当前使用的数据库，需要重新加载当前数据库
        if index == self.get_current_database_index() {
            let current_db = Self::create_database(&app_config.database)?;
            self.current_database = current_db;
        }
        
        Ok(())
    }
    
    /// 保存配置到文件
    pub fn save_config(&self, path: &str) -> Result<()> {
        let config = self.config.lock().unwrap();
        config.save_to_file(path)
            .context("Failed to save configuration")
    }
}