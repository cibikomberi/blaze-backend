use crate::util::validator_util::{validate_password, validate_username};
use crate::util::deserializer_util::{trim_lower, trim};
use serde::Deserialize;
use validator_derive::Validate;

#[derive(Deserialize, Validate)]
pub struct RegisterUserDto {
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