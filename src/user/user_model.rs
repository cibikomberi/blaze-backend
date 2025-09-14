use chrono::Utc;
use diesel::internal::derives::multiconnection::chrono::NaiveDateTime;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Identifiable, Selectable, Serialize, Deserialize, Queryable, Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub username: String,
    pub password: Option<String>,
    pub is_verified: bool,

    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub image: Option<String>,
}

impl User {
    pub fn new(name: String, username: String, email: String, password: Option<String>, image: Option<String>) -> User {
        User {
            id: Uuid::now_v7(),
            name,
            email,
            username,
            password,
            image,
            is_verified: false,
            created_at:  Utc::now().naive_utc(),
            updated_at: None,
        }
    }
}

#[derive(Identifiable, Selectable, Serialize, Deserialize, Queryable, Insertable, Associations, Debug)]
#[diesel(table_name = crate::schema::user_session)]
#[diesel(belongs_to(User))]
pub struct UserSession {
    pub id: Uuid,
    pub jti: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl UserSession {
    pub fn new(jti: Uuid, user_id: Uuid) -> Self {
        UserSession {
            id: Uuid::now_v7(),
            jti,
            user_id,
            created_at: Utc::now().naive_utc(),
            updated_at: None,
        }
    }
}