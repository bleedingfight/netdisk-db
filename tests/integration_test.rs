//! 集成测试 - 测试所有模块功能

use netdisk_db::prelude::*;
use netdisk_db::services::database::sqlite::SqliteDatabase;
use netdisk_db::services::database::connector::DatabaseConnectorFactory;

#[test]
fn test_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.database.db_type, "sqlite");
    assert_eq!(config.database.connection_string, "file_search.db");
    assert_eq!(config.window_width, 800);
    assert_eq!(config.window_height, 600);
}

#[test]
fn test_file_record_creation() {
    let record = FileRecord {
        id: 1,
        name: "test.txt".to_string(),
        path: "/home/user/test.txt".to_string(),
        size: 1024,
        modified_time: "2024-01-01 12:00:00".to_string(),
        file_type: "text/plain".to_string(),
    };
    
    assert_eq!(record.id, 1);
    assert_eq!(record.name, "test.txt");
    assert_eq!(record.size, 1024);
}

#[test]
fn test_utils_functions() {
    use netdisk_db::utils::common::*;
    
    let timestamp = get_timestamp();
    assert!(timestamp > 0);
    
    assert_eq!(format_file_size(512), "512 B");
    assert_eq!(format_file_size(1024), "1.00 KB");
    assert_eq!(format_file_size(1536), "1.50 KB");
    assert_eq!(format_file_size(1048576), "1.00 MB");
    
    assert!(file_exists("src/lib.rs"));
    assert!(!file_exists("non_existent_file.txt"));
    
    assert_eq!(get_file_extension("test.txt"), Some("txt"));
    assert_eq!(get_file_extension("document.pdf"), Some("pdf"));
    assert_eq!(get_file_extension("no_extension"), None);
}

#[test]
fn test_sqlite_database_creation() {
    // 简单的创建测试，不依赖外部文件
    let result = SqliteDatabase::new(":memory:");
    assert!(result.is_ok(), "Failed to create SQLite database: {:?}", result.err());
}

#[test]
fn test_database_connector_factory() {
    // 测试创建 SQLite 连接器
    let sqlite_connector = DatabaseConnectorFactory::create_connector("sqlite");
    assert!(sqlite_connector.is_ok());
    assert_eq!(sqlite_connector.unwrap().get_db_type(), "sqlite");
    
    // 测试创建 MySQL 连接器
    let mysql_connector = DatabaseConnectorFactory::create_connector("mysql");
    assert!(mysql_connector.is_ok());
    assert_eq!(mysql_connector.unwrap().get_db_type(), "mysql");
    
    // 测试不支持的数据库类型
    let invalid_connector = DatabaseConnectorFactory::create_connector("invalid");
    assert!(invalid_connector.is_err());
}