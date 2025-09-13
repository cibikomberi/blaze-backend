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

#[derive(Deserialize, Debug)]
pub struct FileDto {
    pub organization_name: String,
    pub bucket_name: String,
    pub file_path: String,
}

#[derive(Deserialize, Debug)]
pub struct FileQueryDto {
    pub expiry: Option<u64>,
    pub secret_id: String,
    pub signature: String,
}