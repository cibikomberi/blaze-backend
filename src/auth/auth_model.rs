use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Claims {
    pub sub: Uuid,
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
}