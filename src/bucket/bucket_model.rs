use chrono::{NaiveDateTime, Utc};
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::buckets;
use crate::user::user_model::User;
use crate::organization::organization_model::Organization;

#[derive(Insertable, Identifiable, Selectable, Queryable, Associations, Serialize, Debug)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(belongs_to(Organization))]
#[diesel(table_name = buckets)]
pub struct Bucket {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub visibility: BucketVisibility
}

#[derive(DbEnum, Deserialize, Serialize, PartialEq, Debug)]
#[db_enum(existing_type_path = "crate::schema::sql_types::BucketVisibility")]
pub enum BucketVisibility {
    PUBLIC,
    PRIVATE,
}

#[derive(AsChangeset)]
#[diesel(table_name = buckets)]
pub struct BucketChangeset {
    pub name: Option<String>,
    pub visibility: Option<BucketVisibility>
}

impl Bucket {
    pub fn new(name: String, organization_id: Uuid, user_id: Uuid) -> Self {
        Bucket { 
            id: Uuid::now_v7(),
            name,
            organization_id,
            created_by: user_id,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
            visibility: BucketVisibility::PRIVATE,
        }
    }
}