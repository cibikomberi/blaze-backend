use actix_web::post;
use actix_web::web::{Json, ServiceConfig};
use crate::auth::auth_dto::LoginDto;
use crate::auth::auth_service;
use crate::error::ApiError;

#[post("login")]
async fn login(dto: Json<LoginDto>) -> Result<String, ApiError> {
    let LoginDto { username, password } = dto.into_inner();
    auth_service::login(username, password).await
}

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(login);
}