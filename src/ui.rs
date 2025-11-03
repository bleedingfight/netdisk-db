use crate::database::FileRecord;
use slint::ModelRc;
slint::include_modules!();

pub fn file_records_to_model(file_records: Vec<FileRecord>) -> ModelRc<FileItem> {
    let items: Vec<FileItem> = file_records
        .into_iter()
        .map(|record| FileItem {
            id: record.id as i32,
            name: record.name.into(),
            path: record.path.into(),
            size: record.size as i32,
            modified_time: record.modified_time.into(),
            file_type: record.file_type.into(),
        })
        .collect();

    ModelRc::new(slint::VecModel::from(items))
}
