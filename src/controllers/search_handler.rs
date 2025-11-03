//! 搜索处理模块 - 支持字段选择的搜索功能
//!
//! 提供高级搜索功能，支持按特定字段搜索

use crate::models::database::Database;
use crate::views::ui::{file_records_to_model, AppWindow};
use slint::{ModelRc, VecModel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error};

/// 处理高级搜索请求（支持字段选择）
/// 
/// # Arguments
/// * `query` - 搜索关键词
/// * `field` - 搜索字段（可选，None表示搜索所有字段）
/// * `ui` - UI 弱引用
/// * `database` - 数据库实例
/// * `last_search_time` - 上次搜索时间（用于防抖）
/// * `search_delay` - 搜索延迟时间
pub fn handle_advanced_search_request(
    query: &str,
    field: Option<&str>,
    ui: &slint::Weak<AppWindow>,
    database: Arc<Mutex<dyn Database>>,
    last_search_time: Arc<Mutex<Instant>>,
    search_delay: Duration,
) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    // 防抖检查
    let now = Instant::now();
    let mut last_time = last_search_time.lock().unwrap();

    if now.duration_since(*last_time) < search_delay {
        return;
    }

    *last_time = now;
    drop(last_time);

    // 空查询处理
    if query.trim().is_empty() {
        let file_items = ModelRc::new(VecModel::default());
        ui.set_file_items(file_items);
        return;
    }

    // 执行搜索
    let results = if let Some(field_name) = field {
        debug!("Searching field '{}' with query: {}", field_name, query);
        database.lock().unwrap().search_field(field_name, query)
    } else {
        debug!("Searching all fields with query: {}", query);
        database.lock().unwrap().search_files(query)
    };
    
    match results {
        Ok(results) => {
            debug!("Search returned {} results", results.len());
            let file_items = file_records_to_model(results);
            ui.set_file_items(file_items);
        }
        Err(e) => {
            error!("Search failed: {}", e);
            ui.set_file_items(ModelRc::new(VecModel::default()));
        }
    }
}

/// 更新搜索字段列表
/// 
/// # Arguments
/// * `ui` - UI 弱引用
/// * `database` - 数据库实例
pub fn update_search_fields(
    ui: &slint::Weak<AppWindow>,
    database: Arc<Mutex<dyn Database>>,
) {
    let ui = match ui.upgrade() {
        Some(u) => u,
        None => return,
    };

    let fields = database.lock().unwrap().get_search_fields();
    let field_model = slint::ModelRc::new(slint::VecModel::from(
        fields.into_iter().collect::<Vec<_>>()
    ));
    
    ui.set_search_fields(field_model);
    ui.set_current_search_field_index(0); // 重置为第一个字段
    
    debug!("Updated search fields: {:?}", fields);
}