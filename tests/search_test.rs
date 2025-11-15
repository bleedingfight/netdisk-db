//! 搜索功能集成测试

use netdisk_db::services::database::sqlite::SqliteDatabase;
use netdisk_db::models::database::Database;

#[test]
fn test_search_functionality() {
    // 初始化日志
    let _ = tracing_subscriber::fmt::try_init();
    
    println!("测试搜索功能...");
    
    // 创建内存数据库进行测试
    let db = SqliteDatabase::new(":memory:").expect("Failed to create database");
    db.init_database().expect("Failed to initialize database");
    
    // 测试搜索空数据库
    let results = db.search_files("test").expect("Search failed");
    assert_eq!(results.len(), 0, "Empty database should return no results");
    
    println!("✓ 空数据库搜索测试通过");
    
    // 测试字段搜索
    let field_results = db.search_field("name", "test").expect("Field search failed");
    assert_eq!(field_results.len(), 0, "Empty database field search should return no results");
    
    println!("✓ 空数据库字段搜索测试通过");
    
    println!("所有搜索测试通过！");
}

#[test]
fn test_database_connection() {
    let _ = tracing_subscriber::fmt::try_init();
    
    // 测试视频数据库连接（如果存在）
    if std::path::Path::new("video.db").exists() {
        let db = SqliteDatabase::new("video.db").expect("Failed to connect to video database");
        let results = db.search_files("mp4").expect("Search in video database failed");
        println!("视频数据库搜索测试: 找到 {} 个结果", results.len());
        // results.len() 总是 >= 0，所以不需要这个断言
        println!("视频数据库搜索测试通过");
    } else {
        println!("视频数据库不存在，跳过连接测试");
    }
}