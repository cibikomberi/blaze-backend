use crate::config::db_config;
use crate::error::ApiError;
use crate::schema::users::dsl::users;
use crate::schema::users::username;
use crate::user::user_model::User;
use crate::util::jwt_util;
use actix_web::http::StatusCode;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub async fn login(uname: String, password: String) -> Result<String, ApiError>  {
    let pool = db_config::get_connection_pool().await;
    let mut conn = pool.get().await?;
    let user = users
        .filter(username.eq(uname))
        .first::<User>(&mut conn)
        .await?;
        // .first::<User>(&mut conn)
    match bcrypt::verify(password, &user.password) {
        Ok(is_verified) if is_verified => (),
        _ => return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string())),
    };

    match jwt_util::issue(user.id) {
        Ok(token) => Ok(token),
        _ => Err(ApiError::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    }
}