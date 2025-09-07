use actix_web::http::StatusCode;
use diesel::associations::HasTable;
use diesel::pg::Pg;
use diesel::{debug_query, BoolExpressionMethods, QueryDsl};
use crate::config::db_config;
use crate::error::ApiResponse;
use crate::schema::users::dsl::users;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use diesel::ExpressionMethods;
use crate::user::user_model::User;
use diesel::PgTextExpressionMethods;

pub async fn register_user(name: String, email: String, username: String, raw_password: String) -> Result<User, ApiResponse> {
    let password = bcrypt::hash(&raw_password, 10).unwrap();
    let pool = db_config::get_connection_pool().await;
    let mut conn = pool.get().await.unwrap();

    // let mut conn = db_config::connection().await?;

    Ok(diesel::insert_into(users)
        .values(User::new(name, username, email, password))
        .get_result::<User>(&mut conn).await
        .map_err(|_| ApiResponse::new(StatusCode::CONFLICT, "frokpskf".to_string()))?)
}

pub async fn get() -> Result<Vec<User>, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let users_list = users::table()
        .get_results::<User>(&mut conn)
        .await?;

    Ok(users_list) 
}

pub async fn find_by_id(user_id: Uuid) -> Option<User> {
    let mut conn = db_config::get_connection().await.ok()?;

    let user = users.find(user_id).first(&mut conn).await.ok()?;
    Some(user)
}

pub async fn search(keyword: String, limit: i64, cursor: Option<Uuid>) -> Result<Vec<User>, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    
    
    let mut query = users
        .filter(crate::schema::users::username.ilike(format!("%{}%", keyword))
                .or(crate::schema::users::name.ilike(format!("%{}%", keyword)))
                .or(crate::schema::users::email.ilike(format!("%{}%", keyword))))
            .order(crate::schema::users::id.asc()) // 👈 must match cursor logic
        .into_boxed();
    if let Some(cursor) = cursor {
        query = query.filter(crate::schema::users::id.gt(cursor));
    }
    let sql = debug_query::<Pg, _>(&query);
println!("SQL: {}", sql); 
    let results = query
        .limit(limit as i64)
        .load::<User>(&mut conn)
        .await?;
    
    Ok(results)
}
