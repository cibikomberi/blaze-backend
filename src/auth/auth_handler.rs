use crate::auth::auth_dto::{CodeDto, LoginDto, TokenDto};
use crate::auth::auth_middleware::jwt_auth;
use crate::auth::auth_service;
use crate::error::ApiResponse;
use crate::user::user_model::User;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use actix_web::middleware::from_fn;
use actix_web::web::{Json, Query, Redirect, ServiceConfig};
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};

#[post("login")]
async fn login(dto: Json<LoginDto>) -> HttpResponse {
    let LoginDto { username, password } = dto.into_inner();
    let (token, refresh_toke) = auth_service::login(username, password).await.unwrap();
    let cookie = Cookie::build("refresh_token", refresh_toke)
        .path("/api/auth/refresh_token")
        .http_only(true)
        .secure(true)
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

#[get("google")]
async fn google_auth() -> impl Responder {
    Redirect::to(auth_service::google_redirect_url())
}
#[get("google/callback")]
async fn google_callback(code: Query<CodeDto>) -> HttpResponse {
    let CodeDto { code } = code.into_inner();
    let (token, refresh_toke) = auth_service::google_oauth(code).await.unwrap();
    let cookie = Cookie::build("refresh_token", refresh_toke)
        .path("/api/auth/refresh_token")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .finish();
    HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, format!("/auth/success?token={token}")))
        .cookie(cookie)
        .finish()
}

#[get("github")]
async fn github_auth() -> impl Responder {
    Redirect::to(auth_service::github_redirect_url())
}


#[get("github/callback")]
async fn github_callback(code: Query<CodeDto>) -> HttpResponse {
    let CodeDto { code } = code.into_inner();
    let (token, refresh_toke) = auth_service::github_oauth(code).await.unwrap();
    let cookie = Cookie::build("refresh_token", refresh_toke)
        .path("/api/auth/refresh_token")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .finish();
    HttpResponse::TemporaryRedirect()
        .append_header((LOCATION, format!("/auth/success?token={token}")))
        .cookie(cookie)
        .finish()
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
    cfg.service(google_auth);
    cfg.service(github_auth);
    cfg.service(refresh_token);

    cfg.service(web::scope("")
        .wrap(from_fn(jwt_auth))
        .service(who_am_i));
}
