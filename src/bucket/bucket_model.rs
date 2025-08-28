use chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;
use crate::schema::buckets;
use crate::user::user_model::User;
use crate::organization::organization_model::Organization;

#[derive(Insertable, Identifiable, Selectable, Queryable, Associations)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(belongs_to(Organization))]
#[diesel(table_name = buckets)]
pub struct Bucket {
    id: Uuid,
    name: String,
    organization_id: Uuid,
    created_by: Uuid,
    created_at: NaiveDateTime,
    updated_at: Option<NaiveDateTime>,
}