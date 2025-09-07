use chrono::{NaiveDateTime, Utc};
use diesel::deserialize::FromSqlRow;
use diesel::prelude::QueryableByName;
use diesel::{Associations, Insertable, Queryable, Selectable};
use serde::Serialize;
use uuid::Uuid;
use crate::schema::folders;
use crate::user::user_model::User;
use crate::bucket::bucket_model::Bucket;
// use diesel::alias;

// alias!(folders as parent_folders);
#[derive(QueryableByName, Queryable, Selectable, Insertable, Associations, Serialize, Debug)]
#[diesel(table_name = folders)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(belongs_to(Bucket))]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub bucket_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl Folder {
    pub fn new(name: String, bucket_id: Uuid, parent_id: Option<Uuid>, user_id: Uuid) -> Self {
        Folder {
            id: Uuid::now_v7(),
            name,
            bucket_id,
            parent_id,
            created_by: user_id,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }
    }
}