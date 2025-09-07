use crate::auth::auth_model::{Claims, RefreshTokenClaims};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use lazy_static::lazy_static;
use std::env;
use jsonwebtoken::errors::Error;
use uuid::Uuid;

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    static ref JWT_EXPIRY: i64 = env::var("JWT_EXPIRY").expect("JWT_EXPIRY must be set").parse::<i64>().expect("JWT_EXPIRY must be a number");
    static ref REFRESH_TOKEN_EXPIRY: i64 = env::var("REFRESH_TOKEN_EXPIRY").expect("Refresh token must be set").parse::<i64>().expect("Refresh token must be a number");
    static ref APPLICATION_NAME: String = env::var("APPLICATION_NAME").expect("APPLICATION_NAME must be set");
    static ref ENCODING_KEY: EncodingKey = EncodingKey::from_secret(JWT_SECRET.as_bytes());
    static ref DECODING_KEY: DecodingKey = DecodingKey::from_secret(JWT_SECRET.as_bytes());
}

pub fn issue(user_id: Uuid) -> Result<String, Error> {
    let claims = Claims {
        sub: user_id,
        iss: APPLICATION_NAME.to_string(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + Duration::seconds(JWT_EXPIRY.clone())).timestamp(),
    };

    jsonwebtoken::encode(&Header::default(), &claims, &ENCODING_KEY)
}

pub fn issue_refresh_token(user_id: Uuid, jti: Uuid) -> Result<String, Error> {
    let claims = RefreshTokenClaims {
        sub: user_id,
        jti,
        iss: APPLICATION_NAME.to_string(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + Duration::seconds(REFRESH_TOKEN_EXPIRY.clone())).timestamp(),
    };
    let token = jsonwebtoken::encode(&Header::default(), &claims, &ENCODING_KEY)?;
    Ok(token)

}

pub fn decode(token: &str) -> Result<TokenData<Claims>, Error> {
    jsonwebtoken::decode::<Claims>(token, &DECODING_KEY, &Validation::default())
}

pub fn decode_refresh_token(token: &str) -> Result<TokenData<RefreshTokenClaims>, Error> {
    jsonwebtoken::decode::<RefreshTokenClaims>(token, &DECODING_KEY, &Validation::default())
}