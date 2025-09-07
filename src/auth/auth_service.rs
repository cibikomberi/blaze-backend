use crate::config::db_config;
use crate::error::ApiResponse;
use crate::schema::users::dsl::users;
use crate::schema::users::username;
use crate::user::user_model::{User, UserSession};
use crate::util::jwt_util;
use actix_web::http::StatusCode;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use crate::schema::user_session;

pub async fn login(uname: String, password: String) -> Result<(String, String), ApiResponse>  {
    let pool = db_config::get_connection_pool().await;
    let mut conn = pool.get().await?;
    let user = users
        .filter(username.eq(uname))
        .first::<User>(&mut conn)
        .await?;
    drop(conn);
    match bcrypt::verify(password, &user.password) {
        Ok(is_verified) if is_verified => (),
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string())),
    };

    let token = match jwt_util::issue(user.id) {
        Ok(token) => token,
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };
    let jti = Uuid::now_v7();
    let refresh_token = match jwt_util::issue_refresh_token(user.id, jti) {
        Ok(token) => token,
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };
    let mut conn = db_config::get_connection().await?;
    let _ = diesel::insert_into(user_session::table)
        .values(UserSession::new(jti, user.id))
        .get_result::<UserSession>(&mut conn)
        .await?;

    Ok((token, refresh_token))
}

pub async fn refresh_token(token: String) -> Result<(String, String), ApiResponse> {
    let token_claims = jwt_util::decode_refresh_token(&token).unwrap();
    let jti = token_claims.claims.jti;
    let user_id = token_claims.claims.sub;
    println!("User ID: {}", user_id);
    println!("JTI: {}", jti);
    let mut conn = db_config::get_connection().await?;
    let _ = user_session::table
        .filter(user_session::jti.eq(jti))
        .filter(user_session::user_id.eq(user_id))
        .get_result::<UserSession>(&mut conn)
        .await?;
    let updated_jti = Uuid::now_v7();
    let _ = diesel::update(user_session::dsl::user_session)
        .filter(user_session::jti.eq(jti))
        .filter(user_session::user_id.eq(user_id))
        .set((
            user_session::jti.eq(updated_jti),
            user_session::updated_at.eq(Utc::now().naive_utc())))
        .execute(&mut conn)
        .await?;
    let token = match jwt_util::issue(user_id) {
        Ok(token) => token,
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };
    let refresh_token = match jwt_util::issue_refresh_token(user_id, updated_jti) {
        Ok(token) => token,
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };
    Ok((token, refresh_token))
}