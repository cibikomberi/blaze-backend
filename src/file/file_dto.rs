use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SearchFileDto {
    pub folder_id: Uuid,
    pub keyword: Option<String>,
    pub limit: i64,
    pub cursor: Option<Uuid>
}

#[derive(Deserialize)]
pub struct FileNameDTO {
    pub file_name: String,
}

#[derive(Deserialize)]
pub struct FileIdDto {
    pub file_id: Uuid,
}