use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
}