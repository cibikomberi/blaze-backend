use chrono::Utc;
use crate::config::db_config;
use crate::error::ApiError;
use diesel::internal::derives::multiconnection::chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, RunQueryDsl, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::users::dsl::users;

#[derive(Identifiable, Selectable, Serialize, Deserialize, Queryable, Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub is_verified: bool,

    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl User {
    // pub async  fn find_all() -> Result<Vec<User>, ApiError> {
    //     let mut conn = db_config::connection().await?;

    //     users.load::<User>(&mut conn)
    // }

    pub fn new(name: String, username: String, email: String, password: String) -> User {
        User {
            id: Uuid::now_v7(),
            name,
            email,
            username,
            password,
            is_verified: false,
            created_at:  Utc::now().naive_utc(),
            updated_at: None,
        }
    }
}
