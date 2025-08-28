use actix_web::http::StatusCode;
use diesel::associations::HasTable;
use crate::config::db_config;
use crate::error::ApiError;
use crate::schema::users::dsl::users;
use diesel_async::RunQueryDsl;
use crate::user::user_model::User;

pub async fn register_user(name: String, email: String, username: String, raw_password: String) -> Result<User, ApiError> {
    let password = bcrypt::hash(&raw_password, 10).unwrap();
    let pool = db_config::get_connection_pool().await;
    let mut conn = pool.get().await.unwrap();

    // let mut conn = db_config::connection().await?;

    Ok(diesel::insert_into(users)
        .values(User::new(name, username, email, password))
        .get_result::<User>(&mut conn).await
        .map_err(|_| ApiError::new(StatusCode::CONFLICT, "frokpskf".to_string()))?)
}

pub async fn get() -> Result<Vec<User>, ApiError> {
    let pool = db_config::get_connection_pool().await;
    let mut conn = pool.get().await.unwrap();

    Ok(users::table()
    .get_results::<User>(&mut conn)
    // .load_stream
    // .execute(&mut conn)
    .await?)
    
}