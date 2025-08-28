use chrono::NaiveDateTime;
use diesel::{Associations, Insertable, Queryable, Selectable};
use uuid::Uuid;
use crate::schema::folders;
use crate::user::user_model::User;
use crate::bucket::bucket_model::Bucket;
#[derive(Selectable, Queryable, Insertable, Associations)]
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