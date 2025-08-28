use crate::auth::auth_model::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use lazy_static::lazy_static;
use std::env;
use uuid::Uuid;

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    static ref JWT_EXPIRY: i64 = env::var("JWT_EXPIRY").expect("JWT_EXPIRY must be set").parse::<i64>().expect("JWT_EXPIRY must be a number");
    static ref APPLICATION_NAME: String = env::var("APPLICATION_NAME").expect("APPLICATION_NAME must be set");
}

pub fn issue(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id,
        iss: APPLICATION_NAME.to_string(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + Duration::seconds(JWT_EXPIRY.clone())).timestamp(),
    };

    jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.as_bytes()))
}