//! 配置模型 - 应用程序配置管理
//! 
//! 提供应用程序配置的序列化和反序列化功能

use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, Context};

/// 数据库配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub db_type: String, // "sqlite", "mysql", etc.
    pub connection_string: String,
    pub name: String, // 数据库显示名称
    pub description: Option<String>, // 数据库描述
}

/// 多数据库配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiDatabaseConfig {
    pub databases: Vec<DatabaseConfig>,
    pub default_database: usize, // 默认数据库索引
}

/// Aria2配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aria2Config {
    pub enabled: bool,
    pub rpc_host: String,
    pub rpc_port: u16,
    pub rpc_secret: Option<String>,
    pub download_dir: String,
}

/// 应用程序主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig, // 当前使用的数据库配置
    pub multi_database: MultiDatabaseConfig, // 多数据库配置
    pub aria2: Aria2Config, // Aria2下载配置
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: "sqlite".to_string(),
            connection_string: "file_search.db".to_string(),
            name: "Default Database".to_string(),
            description: Some("Default file search database".to_string()),
        }
    }
}

impl Default for Aria2Config {
    fn default() -> Self {
        Self {
            enabled: true,
            rpc_host: "127.0.0.1".to_string(),
            rpc_port: 6800,
            rpc_secret: None,
            download_dir: "./downloads".to_string(),
        }
    }
}

impl Default for MultiDatabaseConfig {
    fn default() -> Self {
        Self {
            databases: vec![DatabaseConfig::default()],
            default_database: 0,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_db = DatabaseConfig::default();
        let multi_db = MultiDatabaseConfig::default();
        let aria2_config = Aria2Config::default();
        
        Self {
            database: default_db,
            multi_database: multi_db,
            aria2: aria2_config,
            window_width: 800,
            window_height: 600,
        }
    }
}

impl AppConfig {
    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: AppConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    /// 切换到指定数据库
    pub fn switch_database(&mut self, index: usize) -> Result<()> {
        if index >= self.multi_database.databases.len() {
            anyhow::bail!("Database index {} out of range", index);
        }
        
        self.database = self.multi_database.databases[index].clone();
        self.multi_database.default_database = index;
        
        Ok(())
    }

    /// 获取当前数据库索引
    pub fn current_database_index(&self) -> usize {
        self.multi_database.default_database
    }

    /// 获取数据库列表
    pub fn database_list(&self) -> &Vec<DatabaseConfig> {
        &self.multi_database.databases
    }

    /// 添加新数据库配置
    pub fn add_database(&mut self, config: DatabaseConfig) {
        self.multi_database.databases.push(config);
    }

    /// 移除数据库配置
    pub fn remove_database(&mut self, index: usize) -> Result<()> {
        if index >= self.multi_database.databases.len() {
            anyhow::bail!("Database index {} out of range", index);
        }
        
        if self.multi_database.databases.len() <= 1 {
            anyhow::bail!("Cannot remove the last database");
        }
        
        self.multi_database.databases.remove(index);
        
        // 调整默认数据库索引
        if self.multi_database.default_database >= index {
            if self.multi_database.default_database > 0 {
                self.multi_database.default_database -= 1;
            }
            // 更新当前数据库配置
            self.database = self.multi_database.databases[self.multi_database.default_database].clone();
        }
        
        Ok(())
    }
}
