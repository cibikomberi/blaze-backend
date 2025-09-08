use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::StatusCode;
use actix_web::middleware::from_fn;
use actix_web::web::{Json, ServiceConfig};
use crate::auth::auth_dto::{LoginDto, TokenDto};
use crate::auth::auth_middleware::jwt_auth;
use crate::auth::auth_service;
use crate::error::ApiResponse;
use crate::user::user_model::User;

#[post("login")]
async fn login(dto: Json<LoginDto>) -> HttpResponse {
    let LoginDto { username, password } = dto.into_inner();
    let (token, refresh_toke) = auth_service::login(username, password).await.unwrap();
    let cookie = Cookie::build("refresh_token", refresh_toke)
        .path("/api/auth/refresh_token")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .finish();
    HttpResponse::Ok().cookie(cookie)
        .content_type("application/json")
        .json(Json(TokenDto{ token }))
}

#[post("refresh_token")]
async fn refresh_token(request: HttpRequest) -> HttpResponse {
    match request.cookie("refresh_token") {
        Some(refresh_token) => {
            let (token, refresh_token) = auth_service::refresh_token(refresh_token.value().to_string()).await.unwrap();
            let cookie = Cookie::build("refresh_token", refresh_token)
                .path("/api/auth/refresh_token")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .finish();
            HttpResponse::Ok().cookie(cookie)
            .content_type("application/json")
            .json(Json(TokenDto{ token }))
        },
        _ => {HttpResponse::BadRequest().finish()}
    }
}

#[get("me")]
async fn who_am_i(request: HttpRequest) -> Result<impl Responder, ApiResponse> {
    if let Some(user) = request.extensions().get::<User>() {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    }
}

pub fn auth_routes(cfg: &mut ServiceConfig) {
    cfg.service(login);
    cfg.service(refresh_token);

    cfg.service(web::scope("")
        .wrap(from_fn(jwt_auth))
        .service(who_am_i));
}
