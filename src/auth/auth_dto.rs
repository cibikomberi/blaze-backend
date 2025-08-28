use crate::util::deserializer_util::{trim, trim_lower};
use crate::util::validator_util::validate_password;
use serde::Deserialize;
use validator_derive::Validate;
#[derive(Deserialize, Validate)]
pub struct LoginDto {
    #[validate(length(min = 4, max = 32))]
    #[serde(deserialize_with = "trim_lower")]
    pub username: String,

    #[serde(deserialize_with = "trim")]
    #[validate(length(min = 8, max = 24))]
    #[validate(custom(function = "validate_password"))]
    pub password: String,
}