use crate::user::user_model::User;
use crate::util::validator_util::{validate_password, validate_username};
use crate::util::deserializer_util::{trim_lower, trim};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct  RegisterUserDto {
    #[validate(length(min = 4, max = 32))]
    pub name: String,

    #[validate(email)]
    #[serde(deserialize_with = "trim_lower")]
    pub email: String,

    #[validate(length(min = 4, max = 32))]
    #[serde(deserialize_with = "trim_lower")]
    #[validate(custom(function = "validate_username"))]
    pub username: String,

    #[serde(deserialize_with = "trim")]
    #[validate(length(min = 8, max = 24))]
    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

#[derive(Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub username: String,
    pub is_verified: bool
}

impl From<User> for UserDto {
    fn from(user: User) -> UserDto {
        UserDto {
            id: user.id,
            name: user.name,
            email: user.email,
            username: user.username,
            is_verified: user.is_verified
        }
    }
}

// #[derive(Deserialize, Validate)]
// pub struct PaginationDto {
// ,
// }

#[derive(Deserialize)]
pub struct SearchDto {
    pub keyword: String,
    pub limit: i64,
    pub cursor: Option<Uuid>,
}