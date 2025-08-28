use crate::error::ApiError;
use crate::user::user_dto::RegisterUserDto;
use crate::user::user_model::User;
use crate::user::user_service::{self, register_user};
use actix_web::web::Json;
use actix_web::{get, post, web};
use actix_web::http::StatusCode;
use validator::Validate;

#[post("")]
async fn register(dto: Json<RegisterUserDto>) -> Result<Json<User>, ApiError> {
    if let Err(e) = dto.validate() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, e.to_string()));
    }
    let RegisterUserDto{ name, email, username, password } = dto.into_inner();
    Ok(Json(register_user(name, email, username, password).await?))
}

#[get("")]
async fn get()  -> Result<Json<Vec<User>>, ApiError> {
    Ok(Json(user_service::get().await?))
}
pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get);
    cfg.service(register);
}