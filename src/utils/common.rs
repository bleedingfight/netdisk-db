//! 通用工具函数模块
//! 
//! 包含项目中使用的各种工具函数

use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// 获取当前时间戳（秒）
/// 
/// # Returns
/// * `u64` - Unix时间戳
/// 
/// # Panics
/// 如果系统时间早于Unix纪元（1970年）
pub fn get_timestamp() -> u64 {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    debug!("Generated timestamp: {}", timestamp);
    timestamp
}

/// 格式化文件大小为人类可读格式
/// 
/// # Arguments
/// * `size` - 文件大小（字节）
/// 
/// # Returns
/// * `String` - 格式化后的大小字符串（如 "1.50 MB"）
pub fn format_file_size(size: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    // 转换到合适的单位
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    // 格式化输出
    let result = if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    };
    
    debug!("Formatted file size: {} bytes -> {}", size, result);
    result
}

/// 检查文件是否存在
/// 
/// # Arguments
/// * `path` - 文件路径
/// 
/// # Returns
/// * `bool` - 文件是否存在
pub fn file_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

/// 获取文件扩展名
/// 
/// # Arguments
/// * `filename` - 文件名
/// 
/// # Returns
/// * `Option<&str>` - 文件扩展名（不包含点）
pub fn get_file_extension(filename: &str) -> Option<&str> {
    std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
}