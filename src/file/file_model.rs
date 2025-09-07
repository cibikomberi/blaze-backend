use chrono::{NaiveDateTime, Utc};
use diesel::{Associations, Insertable, Queryable, Selectable};
use serde::Serialize;
use uuid::Uuid;
use crate::schema::files;
use crate::user::user_model::User;
use crate::folder::folder_model::Folder;
#[derive(Selectable, Queryable, Insertable, Associations, Serialize, Debug)]
#[diesel(table_name = files)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(belongs_to(Folder))]
pub struct File {
    pub id: Uuid,
    pub name: String,
    pub folder_id: Uuid,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl File {
    pub fn new(name: String, folder_id: Uuid, created_by: Uuid) -> Self {
        File {
            id: Uuid::now_v7(),
            name,
            folder_id,
            created_by,
            created_at: Utc::now().naive_utc(),
            updated_at: None
        }
    }
}