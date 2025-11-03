//! SQLite 数据库服务实现
//! 
//! 提供 SQLite 数据库的具体实现

use anyhow::{Result, Context};
use rusqlite::params;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::models::database::{Database, FileRecord};

/// SQLite 数据库连接池包装器
pub struct SqliteDatabase {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteDatabase {
    /// 创建新的 SQLite 数据库实例
    /// 
    /// # Arguments
    /// * `db_path` - 数据库文件路径
    pub fn new(db_path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .build(manager)
            .context("Failed to create connection pool")?;
        
        Ok(Self { pool })
    }
}

impl Database for SqliteDatabase {
    fn init_database(&self) -> Result<()> {
        let conn = self.pool.get()
            .context("Failed to get connection from pool")?;
        
        // 创建文件表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                size INTEGER NOT NULL,
                modified_time TEXT NOT NULL,
                file_type TEXT NOT NULL
            )",
            [],
        ).context("Failed to create files table")?;

        // 创建索引优化搜索性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_name ON files(name)",
            [],
        ).context("Failed to create index on files.name")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)",
            [],
        ).context("Failed to create index on files.path")?;

        // 如果表为空，添加示例数据
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row| row.get(0),
        ).context("Failed to count files")?;

        if count == 0 {
            self.add_sample_data()?;
        }

        Ok(())
    }

    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>> {
        let search_pattern = format!("%{}%", query);
        
        let conn = self.pool.get()
            .context("Failed to get connection from pool")?;
        
        let mut stmt = conn.prepare(
            "SELECT id, name, path, size, modified_time, file_type
             FROM files
             WHERE name LIKE ?1 OR path LIKE ?1
             ORDER BY name
             LIMIT 100"
        ).context("Failed to prepare search statement")?;

        let file_iter = stmt.query_map(params![search_pattern], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get(3)?,
                modified_time: row.get(4)?,
                file_type: row.get(5)?,
            })
        }).context("Failed to execute search query")?;

        let mut results = Vec::new();
        for file in file_iter {
            results.push(file.context("Failed to map file record")?);
        }

        Ok(results)
    }

    fn search_field(&self, field: &str, query: &str) -> Result<Vec<FileRecord>> {
        let search_pattern = format!("%{}%", query);
        
        let conn = self.pool.get()
            .context("Failed to get connection from pool")?;
        
        // 验证字段名以防止SQL注入
        let valid_fields = ["name", "path", "size", "modified_time", "file_type"];
        if !valid_fields.contains(&field) {
            anyhow::bail!("Invalid field name: {}", field);
        }
        
        let sql = format!(
            "SELECT id, name, path, size, modified_time, file_type
             FROM files
             WHERE {} LIKE ?1
             ORDER BY name
             LIMIT 100",
            field
        );
        
        let mut stmt = conn.prepare(&sql)
            .context("Failed to prepare search statement")?;

        let file_iter = stmt.query_map(params![search_pattern], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get(3)?,
                modified_time: row.get(4)?,
                file_type: row.get(5)?,
            })
        }).context("Failed to execute search query")?;

        let mut results = Vec::new();
        for file in file_iter {
            results.push(file.context("Failed to map file record")?);
        }

        Ok(results)
    }

    fn get_search_fields(&self) -> Vec<String> {
        vec!["name".to_string(), "path".to_string(), "size".to_string(), "modified_time".to_string(), "file_type".to_string()]
    }
}

impl SqliteDatabase {
    /// 添加示例数据到数据库
    fn add_sample_data(&self) -> Result<()> {
        let conn = self.pool.get()
            .context("Failed to get connection from pool")?;
        
        let sample_files = [
            ("document.txt", "/home/user/documents/document.txt", 1024, "2024-01-15 10:30:00", "text/plain"),
            ("report.pdf", "/home/user/documents/report.pdf", 2048, "2024-01-14 15:45:00", "application/pdf"),
            ("image.jpg", "/home/user/pictures/image.jpg", 3072, "2024-01-13 08:20:00", "image/jpeg"),
            ("data.csv", "/home/user/data/data.csv", 512, "2024-01-12 14:15:00", "text/csv"),
            ("presentation.pptx", "/home/user/presentations/presentation.pptx", 4096, "2024-01-11 16:30:00", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
        ];

        for (name, path, size, modified_time, file_type) in &sample_files {
            conn.execute(
                "INSERT INTO files (name, path, size, modified_time, file_type) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![name, path, size, modified_time, file_type],
            ).context("Failed to insert sample data")?;
        }

        Ok(())
    }
}