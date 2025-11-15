//! 搜索功能测试程序

use netdisk_db::services::database::sqlite::SqliteDatabase;
use netdisk_db::models::database::Database;

fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("测试搜索功能...");
    
    // 创建数据库实例
    let db = SqliteDatabase::new("video.db")?;
    db.init_database()?;
    
    // 测试搜索
    println!("测试搜索 'mp4':");
    let results = db.search_files("mp4")?;
    println!("找到 {} 个结果", results.len());
    
    for (i, record) in results.iter().take(3).enumerate() {
        println!("结果 {}: name={}, path={}, size={}", 
                i + 1, record.name, record.path, record.size);
    }
    
    // 测试字段搜索
    println!("\n测试按名称字段搜索 '特警':");
    let field_results = db.search_field("name", "特警")?;
    println!("找到 {} 个结果", field_results.len());
    
    for (i, record) in field_results.iter().take(3).enumerate() {
        println!("结果 {}: name={}, path={}, size={}", 
                i + 1, record.name, record.path, record.size);
    }
    
    println!("\n测试完成！");
    Ok(())
}