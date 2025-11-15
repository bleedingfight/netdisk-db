//! SQLite 数据库服务实现
//!
//! 提供 SQLite 数据库的具体实现

use crate::models::database::{Database, FileRecord};
use anyhow::{Context, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use tracing::debug;

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
        let conn = self
            .pool
            .get()
            .context("Failed to get connection from pool")?;

        debug!("开始初始化数据库...");

        // 创建文件表
        debug!("创建 video 表...");
        match conn.execute(
            "CREATE TABLE IF NOT EXISTS video (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                size INTEGER NOT NULL,
                etag TEXT NOT NULL,
                modified_time INTEGER NOT NULL,
                file_type TEXT NOT NULL
            )",
            [],
        ) {
            Ok(_) => debug!("video 表创建成功"),
            Err(e) => {
                debug!("video 表创建失败: {}", e);
                return Err(anyhow::anyhow!("Failed to create video table: {}", e));
            }
        }

        // 创建索引优化搜索性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_video_name ON video(name)",
            [],
        )
        .context("Failed to create index on video.name")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_video_path ON video(path)",
            [],
        )
        .context("Failed to create index on video.path")?;

        // 如果表为空，添加示例数据
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM video", [], |row| row.get(0))
            .context("Failed to count video files")?;

        if count == 0 {
            debug!("表为空，添加示例数据...");
            // 使用同一个连接添加示例数据，对于内存数据库很重要
            Self::add_sample_data_with_conn(&conn)?;
        }

        debug!("数据库初始化完成");
        Ok(())
    }

    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>> {
        let search_pattern = format!("%{}%", query);

        let conn = self
            .pool
            .get()
            .context("Failed to get connection from pool")?;

        let command = "SELECT id, path, size, etag, modified_time, file_type, name FROM video where path like ? limit 100";
        let mut stmt = conn
            .prepare(&command)
            .context("Failed to prepare search statement")?;

        debug!("执行命令:{}", &command);
        let file_iter = stmt
            .query_map(params![search_pattern], |row| {
                // 获取所有字段的原始值用于调试
                let id: i64 = row.get(0)?;
                let path: String = row.get(1)?;
                
                // 安全地获取 size 字段，作为 u64 获取
                let size_value: Result<i64, _> = row.get(2);
                let size = match size_value {
                    Ok(s) => {
                        debug!("Got size as i64: {} for file: {}", s, path);
                        if s < 0 {
                            debug!("Negative size detected: {}, converting to positive", s);
                            s as u64
                        } else {
                            s as u64
                        }
                    }
                    Err(e) => {
                        debug!("Failed to get size for file {}: {}, using 0", path, e);
                        0u64
                    }
                };
                
                let etag: String = row.get(3)?;
                
                // 安全地获取 modified_time 字段
                let modified_time_value: Result<i64, _> = row.get(4);
                let modified_time = match modified_time_value {
                    Ok(t) => {
                        debug!("Got modified_time as i64: {} for file: {}", t, path);
                        t
                    }
                    Err(_) => {
                        // 如果无法作为 i64 获取，尝试作为字符串然后解析
                        let time_str: Result<String, _> = row.get(4);
                        match time_str {
                            Ok(s) => {
                                debug!("Got modified_time as string: '{}' for file: {}", s, path);
                                s.parse::<i64>().unwrap_or(0)
                            }
                            Err(e) => {
                                debug!("Failed to get modified_time for file {}: {}, using 0", path, e);
                                0
                            }
                        }
                    }
                };
                
                let file_type: String = row.get(5)?;
                let name: String = row.get(6)?;
                
                debug!("Creating FileRecord: id={}, name={}, path={}, size='{}', etag={}, modified_time={}, file_type={}",
                       id, name, path, size, etag, modified_time, file_type);
                
                Ok(FileRecord {
                    id,
                    path,
                    size,
                    etag,
                    modified_time,
                    file_type,
                    name,
                })
            })
            .context("Failed to execute search query")?;

        let mut results = Vec::new();
        for file in file_iter {
            results.push(file.context("Failed to map file record")?);
        }

        Ok(results)
    }

    fn search_field(&self, field: &str, query: &str) -> Result<Vec<FileRecord>> {
        let search_pattern = format!("%{}%", query);

        let conn = self
            .pool
            .get()
            .context("Failed to get connection from pool")?;

        // 验证字段名以防止SQL注入
        let valid_fields = [
            "id",
            "path",
            "size",
            "etag",
            "modified_time",
            "file_type",
            "name",
        ];
        if !valid_fields.contains(&field) {
            anyhow::bail!("Invalid field name: {}", field);
        }

        let sql = format!(
            "SELECT id, path, size, etag, modified_time, file_type, name
             FROM video
             WHERE {} LIKE ?1
             ORDER BY name
             LIMIT 100",
            field
        );

        let mut stmt = conn
            .prepare(&sql)
            .context("Failed to prepare search statement")?;

        let file_iter = stmt
            .query_map(params![search_pattern], |row| {
                // 获取所有字段的原始值用于调试
                let id: i64 = row.get(0)?;
                let path: String = row.get(1)?;
                
                // 安全地获取 size 字段，作为 u64 获取
                let size_value: Result<i64, _> = row.get(2);
                let size = match size_value {
                    Ok(s) => {
                        debug!("Got size as i64: {} for file: {}", s, path);
                        if s < 0 {
                            debug!("Negative size detected: {}, converting to positive", s);
                            s as u64
                        } else {
                            s as u64
                        }
                    }
                    Err(e) => {
                        debug!("Failed to get size for file {}: {}, using 0", path, e);
                        0u64
                    }
                };
                
                let etag: String = row.get(3)?;
                
                // 安全地获取 modified_time 字段
                let modified_time_value: Result<i64, _> = row.get(4);
                let modified_time = match modified_time_value {
                    Ok(t) => {
                        debug!("Got modified_time as i64: {} for file: {}", t, path);
                        t
                    }
                    Err(_) => {
                        // 如果无法作为 i64 获取，尝试作为字符串然后解析
                        let time_str: Result<String, _> = row.get(4);
                        match time_str {
                            Ok(s) => {
                                debug!("Got modified_time as string: '{}' for file: {}", s, path);
                                s.parse::<i64>().unwrap_or(0)
                            }
                            Err(e) => {
                                debug!("Failed to get modified_time for file {}: {}, using 0", path, e);
                                0
                            }
                        }
                    }
                };
                
                let file_type: String = row.get(5)?;
                let name: String = row.get(6)?;
                
                debug!("Creating FileRecord: id={}, name={}, path={}, size='{}', etag={}, modified_time={}, file_type={}",
                       id, name, path, size, etag, modified_time, file_type);
                
                Ok(FileRecord {
                    id,
                    path,
                    size,
                    etag,
                    modified_time,
                    file_type,
                    name,
                })
            })
            .context("Failed to execute search query")?;

        let mut results = Vec::new();
        for file in file_iter {
            results.push(file.context("Failed to map file record")?);
        }

        Ok(results)
    }

    fn get_search_fields(&self) -> Vec<String> {
        vec![
            "id",
            "path",
            "size",
            "etag",
            "modified_time",
            "file_type",
            "name",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

impl SqliteDatabase {
    /// 添加示例数据到数据库（使用提供的连接）
    fn add_sample_data_with_conn(conn: &rusqlite::Connection) -> Result<()> {
        debug!("开始添加示例数据...");

        let sample_files = [
            (
                "document.txt",
                "/home/user/documents/document.txt",
                1024,
                "2024-01-15 10:30:00",
                "text/plain",
            ),
            (
                "report.pdf",
                "/home/user/documents/report.pdf",
                2048,
                "2024-01-14 15:45:00",
                "application/pdf",
            ),
            (
                "image.jpg",
                "/home/user/pictures/image.jpg",
                3072,
                "2024-01-13 08:20:00",
                "image/jpeg",
            ),
            (
                "data.csv",
                "/home/user/data/data.csv",
                512,
                "2024-01-12 14:15:00",
                "text/csv",
            ),
            (
                "presentation.pptx",
                "/home/user/presentations/presentation.pptx",
                4096,
                "2024-01-11 16:30:00",
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ),
        ];

        for (name, path, size, modified_time, file_type) in &sample_files {
            debug!("插入数据: name={}, path={}, size={}, modified_time={}, file_type={}", name, path, size, modified_time, file_type);
            conn.execute(
                "INSERT INTO video (name, path, size, etag, modified_time, file_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![name, path, size, "sample_etag", modified_time, file_type],
            ).context("Failed to insert sample data")?;
        }

        debug!("示例数据添加完成");
        Ok(())
    }

    /// 添加示例数据到数据库（使用内部连接池）
    #[allow(dead_code)]
    fn add_sample_data(&self) -> Result<()> {
        let conn = self
            .pool
            .get()
            .context("Failed to get connection from pool")?;

        Self::add_sample_data_with_conn(&conn)
    }
}
