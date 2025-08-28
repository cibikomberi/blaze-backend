use chrono::NaiveDateTime;
use diesel::{Associations, Insertable, Queryable, Selectable};
use uuid::Uuid;
use crate::schema::files;
use crate::user::user_model::User;
use crate::folder::folder_model::Folder;
#[derive(Selectable, Queryable, Insertable, Associations)]
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