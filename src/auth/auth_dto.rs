use crate::util::deserializer_util::{trim, trim_lower};
use crate::util::validator_util::validate_password;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
pub struct TokenDto {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CodeDto {
    pub code: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GoogleOauthResponse {
    pub access_token: String,
    pub expires_in: usize,
    pub scope: String,
    pub id_token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GithubOauthResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String
}

#[derive(Deserialize)]
pub struct GoogleUser {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
}

#[derive(Deserialize, Debug)]
pub struct GithubUser {
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub url: String,
}