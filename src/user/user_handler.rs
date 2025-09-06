use crate::error::ApiResponse;
use crate::user::user_dto::{ RegisterUserDto, SearchDto, UserDto};
use crate::user::user_model::User;
use crate::user::user_service::{self, register_user};
use actix_web::middleware::from_fn;
use actix_web::web::{Json, Path, Query};
use actix_web::{get, post, web};
use actix_web::http::StatusCode;
use uuid::Uuid;
use crate::auth::auth_middleware::jwt_auth;
use validator::Validate;

#[post("")]
async fn register(dto: Json<RegisterUserDto>) -> Result<Json<User>, ApiResponse> {
    if let Err(e) = dto.validate() {
        return Err(ApiResponse::new(StatusCode::BAD_REQUEST, e.to_string()));
    }
    let RegisterUserDto{ name, email, username, password } = dto.into_inner();
    Ok(Json(register_user(name, email, username, password).await?))
}

#[get("{id}")]
async fn find(id: Path<Uuid>)  -> Result<Json<UserDto>, ApiResponse> {
    let user = user_service::find_by_id(id.into_inner()).await;
    if let Some(user) = user {
        Ok(Json(UserDto::from(user)))
    } else {
        Err(ApiResponse::new(StatusCode::NOT_FOUND, "User not found".to_string()))
    }
}

#[get("search")]
async fn search(search_dto: Query<SearchDto>)  -> Result<Json<Vec<UserDto>>, ApiResponse> {
    let SearchDto { keyword, limit, cursor } = search_dto.into_inner();
    // let PaginationDto {  } = pagination;
    info!("search keyword: {:?}", keyword);
    let users = user_service::search(keyword, limit, cursor).await?;
    let user_dtos: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();
    Ok(Json(user_dtos))
}
pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register);

    cfg.service(web::scope("")
        .wrap(from_fn(jwt_auth))
        .service(search)
        .service(find));
}