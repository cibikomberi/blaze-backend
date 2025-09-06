use crate::folder::folder_model::Folder;
use chrono::NaiveDateTime;
use diesel::sql_types::{Text, Timestamp, Uuid as SqlUuid};
use diesel::QueryableByName;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateFolderDTO {
    pub name: String,
    pub bucket_id: Uuid,
    pub parent_id: Uuid,
}

#[derive(Deserialize)]
pub struct SearchFolderDto {
    pub bucket_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub keyword: Option<String>,
    pub limit: i64,
    pub cursor: Option<Uuid>,
    #[serde(default)]
    pub cursor_kind: String,
}

#[derive(Serialize)]
pub struct FolderResponseDto {
    pub folder: Folder,
    pub items: Vec<Entry>,
}

#[derive(Deserialize)]
pub struct FolderIdDto {
    pub folder_id: Uuid
}

// #[diesel(check_for_backend(Pg))]
#[derive(QueryableByName, Serialize ,Debug)]
pub struct Entry {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,

    #[diesel(sql_type = Text)]
    pub name: String,

    #[diesel(sql_type = Text)]
    pub kind: String,

    #[diesel(sql_type = Timestamp)]
    pub created_at: NaiveDateTime,

    #[diesel(sql_type = SqlUuid)]
    pub created_by: Uuid,

    #[diesel(sql_type = Text)]
    pub user_name: String,

    #[diesel(sql_type = Text)]
    pub user_email: String,

    #[diesel(sql_type = Text)]
    pub user_username: String,
}