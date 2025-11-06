//! 数据库连接器抽象接口
//!
//! 提供不同类型数据库的连接和数据库列表获取功能

use anyhow::Result;
use crate::models::config::DatabaseConfig;
use std::collections::HashMap;

/// 数据库连接信息
#[derive(Debug, Clone)]
pub struct DatabaseConnectionInfo {
    pub name: String,
    pub db_type: String,
    pub connection_string: String,
    pub description: Option<String>,
}

/// 数据库连接器 trait
///
/// 实现此接口可以为不同的数据库类型提供连接和数据库列表获取功能
pub trait DatabaseConnector: Send + Sync {
    /// 获取数据库类型名称
    fn get_db_type(&self) -> &str;
    
    /// 测试连接是否可用
    fn test_connection(&self, connection_string: &str) -> Result<bool>;
    
    /// 获取数据库列表
    ///
    /// 对于MySQL等服务器数据库，返回服务器上的数据库列表
    /// 对于SQLite等文件数据库，返回指定路径下的数据库文件列表
    fn get_database_list(&self, connection_info: &HashMap<String, String>) -> Result<Vec<DatabaseConnectionInfo>>;
    
    /// 创建数据库配置
    fn create_database_config(&self, name: &str, connection_string: &str, description: Option<String>) -> DatabaseConfig;
}

/// 数据库连接器工厂
pub struct DatabaseConnectorFactory;

impl DatabaseConnectorFactory {
    /// 创建指定类型的数据库连接器
    pub fn create_connector(db_type: &str) -> Result<Box<dyn DatabaseConnector>> {
        match db_type {
            "sqlite" => Ok(Box::new(SqliteConnector::new())),
            "mysql" => Ok(Box::new(MySqlConnector::new())),
            _ => anyhow::bail!("Unsupported database type: {}", db_type),
        }
    }
    
    /// 获取所有支持的连接器
    pub fn get_all_connectors() -> Vec<Box<dyn DatabaseConnector>> {
        vec![
            Box::new(SqliteConnector::new()),
            Box::new(MySqlConnector::new()),
        ]
    }
}

/// SQLite 连接器实现
pub struct SqliteConnector;

impl SqliteConnector {
    pub fn new() -> Self {
        Self
    }
}

impl DatabaseConnector for SqliteConnector {
    fn get_db_type(&self) -> &str {
        "sqlite"
    }
    
    fn test_connection(&self, connection_string: &str) -> Result<bool> {
        // 检查文件是否存在且可读
        let path = std::path::Path::new(connection_string);
        Ok(path.exists() && path.is_file())
    }
    
    fn get_database_list(&self, connection_info: &HashMap<String, String>) -> Result<Vec<DatabaseConnectionInfo>> {
        let mut databases = Vec::new();
        
        // 获取搜索路径，默认为当前目录
        let search_path = connection_info.get("path")
            .map(|s| s.as_str())
            .unwrap_or(".");
        
        let base_path = std::path::Path::new(search_path);
        
        if base_path.exists() && base_path.is_dir() {
            // 读取目录下的所有.db文件
            if let Ok(entries) = std::fs::read_dir(base_path) {
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
                                    
                                    databases.push(DatabaseConnectionInfo {
                                        name: db_name,
                                        db_type: "sqlite".to_string(),
                                        connection_string: db_path,
                                        description: Some(format!("SQLite database file: {}", file_name)),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(databases)
    }
    
    fn create_database_config(&self, name: &str, connection_string: &str, description: Option<String>) -> DatabaseConfig {
        DatabaseConfig {
            db_type: "sqlite".to_string(),
            connection_string: connection_string.to_string(),
            name: name.to_string(),
            description,
        }
    }
}

/// MySQL 连接器实现
pub struct MySqlConnector;

impl MySqlConnector {
    pub fn new() -> Self {
        Self
    }
}

impl DatabaseConnector for MySqlConnector {
    fn get_db_type(&self) -> &str {
        "mysql"
    }
    
    fn test_connection(&self, connection_string: &str) -> Result<bool> {
        // 简单的连接字符串格式验证
        // 格式: mysql://username:password@host:port/database
        Ok(connection_string.starts_with("mysql://") && connection_string.contains('@'))
    }
    
    fn get_database_list(&self, connection_info: &HashMap<String, String>) -> Result<Vec<DatabaseConnectionInfo>> {
        let mut databases = Vec::new();
        
        // 获取连接信息
        let host = connection_info.get("host").map_or("localhost", |v| v.as_str());
        let port = connection_info.get("port").map_or("3306", |v| v.as_str());
        let username = connection_info.get("username").map_or("root", |v| v.as_str());
        let password = connection_info.get("password").map_or("", |v| v.as_str());
        
        // 构建服务器连接字符串（不包含具体数据库）
        let server_connection = format!("mysql://{}:{}@{}:{}", username, password, host, port);
        
        // 这里应该实际连接MySQL服务器并查询数据库列表
        // 由于需要添加mysql依赖，这里先返回模拟数据
        // 实际实现时需要使用 mysql_async 或 similar crate
        
        // 模拟一些常见的数据库名称
        let common_databases = vec!["information_schema", "mysql", "performance_schema", "sys"];
        
        for db_name in common_databases {
            databases.push(DatabaseConnectionInfo {
                name: db_name.to_string(),
                db_type: "mysql".to_string(),
                connection_string: format!("{}/{}", server_connection, db_name),
                description: Some(format!("MySQL database: {}", db_name)),
            });
        }
        
        // 添加一些示例数据库
        databases.push(DatabaseConnectionInfo {
            name: "file_search".to_string(),
            db_type: "mysql".to_string(),
            connection_string: format!("{}/file_search", server_connection),
            description: Some("File search database".to_string()),
        });
        
        databases.push(DatabaseConnectionInfo {
            name: "documents".to_string(),
            db_type: "mysql".to_string(),
            connection_string: format!("{}/documents", server_connection),
            description: Some("Document management database".to_string()),
        });
        
        Ok(databases)
    }
    
    fn create_database_config(&self, name: &str, connection_string: &str, description: Option<String>) -> DatabaseConfig {
        DatabaseConfig {
            db_type: "mysql".to_string(),
            connection_string: connection_string.to_string(),
            name: name.to_string(),
            description,
        }
    }
}