//! UI 视图模块 - 用户界面相关功能
//!
//! 包含 UI 数据转换和界面相关的工具函数

use crate::models::database::FileRecord;
use slint::ModelRc;
use tracing::debug;

// 包含 Slint 生成的模块
slint::include_modules!();

/// 将文件记录列表转换为 UI 模型
///
/// # Arguments
/// * `file_records` - 数据库查询结果
///
/// # Returns
/// * `ModelRc<FileItem>` - Slint UI 模型
// pub fn file_records_to_model(file_records: Vec<FileRecord>) -> ModelRc<FileItem> {
//     debug!(
//         "Converting {} file records to UI model,FileRecord = {:?}",
//         file_records.len(),
//         &file_records[0]
//     );

//     let items: Vec<FileItem> = file_records
//         .into_iter()
//         .map(|record| FileItem {
//             id: record.id as i32,
//             name: record.name.into(),
//             path: record.path.into(),
//             size: record.size as i32,
//             modified_time: record.modified_time.into(),
//             file_type: record.file_type.into(),
//         })
//         .collect();

//     ModelRc::new(slint::VecModel::from(items))
// }
pub fn file_records_to_model(file_records: Vec<FileRecord>) -> ModelRc<FileItem> {
    debug!("Converting {} file records to UI model", file_records.len());

    let items: Vec<FileItem> = file_records
        .into_iter()
        .map(|record| {
            debug!(
                "Processing record: name=[{}], path=[{}], size=[{}], etag=[{}]",
                record.name, record.path, record.size, record.etag
            );

            let final_size = record.size.to_string().into();

            FileItem {
                id: record.id as i32,
                path: record.path.into(),
                size: final_size,
                etag: record.etag.into(),
                modified_time: record.modified_time as i32,
                file_type: record.file_type.into(),
                name: record.name.into(),
            }
        })
        .collect();

    ModelRc::new(slint::VecModel::from(items))
}

/// 将数据库信息列表转换为字符串数组供 ComboBox 使用
///
/// # Arguments
/// * `database_list` - 数据库信息列表 (name, db_type, index)
///
/// # Returns
/// * `ModelRc<string>` - Slint UI 字符串模型
pub fn database_list_to_string_model(
    database_list: Vec<(String, String, usize)>,
) -> ModelRc<slint::SharedString> {
    debug!(
        "Converting {} databases to string model for ComboBox",
        database_list.len()
    );

    let items: Vec<slint::SharedString> = database_list
        .into_iter()
        .map(|(name, db_type, _index)| slint::SharedString::from(format!("{} ({})", name, db_type)))
        .collect();

    ModelRc::new(slint::VecModel::from(items))
}
