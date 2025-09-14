use crate::config::db_config;
use crate::error::ApiResponse;
use crate::schema::users;
use crate::schema::users::username;
use crate::user::user_model::{User, UserSession};
use crate::util::jwt_util;
use actix_web::http::StatusCode;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lazy_static::lazy_static;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use uuid::Uuid;
use crate::auth::auth_dto::{GithubOauthResponse, GithubUser, GoogleOauthResponse, GoogleUser};
use crate::schema::user_session;

lazy_static! {
    static ref GOOGLE_CLIENT_ID: String = std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap().to_string();
    static ref GOOGLE_CLIENT_SECRET: String = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap().to_string();
    static ref GOOGLE_REDIRECT_URI: String = std::env::var("GOOGLE_REDIRECT_URI").unwrap().to_string();
    static ref GOOGLE_SCOPE: String = "https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile".to_string();
    static ref GOOGLE_AUTH_ENDPOINT: String = "https://accounts.google.com/o/oauth2/v2/auth".to_string();

    static ref GITHUB_CLIENT_ID: String = std::env::var("GITHUB_OAUTH_CLIENT_ID").unwrap().to_string();
    static ref GITHUB_CLIENT_SECRET: String = std::env::var("GITHUB_OAUTH_CLIENT_SECRET").unwrap().to_string();
    static ref GITHUB_REDIRECT_URI: String = std::env::var("GITHUB_REDIRECT_URI").unwrap().to_string();
    static ref GITHUB_AUTH_ENDPOINT: String = "https://github.com/login/oauth/authorize".to_string();
    static ref GITHUB_SCOPE: String = "user".to_string();
}
pub async fn login(uname: String, password: String) -> Result<(String, String), ApiResponse>  {
    let pool = db_config::get_connection_pool().await;
    let mut conn = pool.get().await?;
    let user = users::table
        .filter(username.eq(uname))
        .first::<User>(&mut conn)
        .await?;
    drop(conn);
    let actual_password = &user.password.clone().ok_or(ApiResponse::new(StatusCode::FORBIDDEN, "Password login is not available".to_string()))?;
    match bcrypt::verify(password, actual_password) {
        Ok(is_verified) if is_verified => (),
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string())),
    };

    let mut conn = db_config::get_connection().await?;
    create_token(user, &mut conn).await
}

pub fn google_redirect_url() -> String {
    let client_id = &*GOOGLE_CLIENT_ID;
    let redirect_uri = &*GOOGLE_REDIRECT_URI;
    let scope = &*GOOGLE_SCOPE;
    let endpoint = &*GOOGLE_AUTH_ENDPOINT;
    format!("{endpoint}?client_id={client_id}&redirect_uri={redirect_uri}&scope={scope}&response_type=code")
}

pub fn github_redirect_url() -> String {
    let github_redirect_uri = &*GITHUB_REDIRECT_URI;
    let github_client_id = &*GITHUB_CLIENT_ID;
    let scope = &*GITHUB_SCOPE;
    let endpoint = &*GITHUB_AUTH_ENDPOINT;
    format!("{endpoint}?client_id={github_client_id}&redirect_uri={github_redirect_uri}&scope={scope}")
}

pub async fn google_oauth(code: String) -> Result<(String, String), ApiResponse> {
    let form = reqwest::multipart::Form::new()
        .text("client_id", &*GOOGLE_CLIENT_ID)
        .text("client_secret", &*GOOGLE_CLIENT_SECRET)
        .text("redirect_uri", &*GOOGLE_REDIRECT_URI)
        .text("grant_type", "authorization_code")
        .text("code", code);

    let res = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .multipart(form)
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send().await.unwrap()
        // .json::<GoogleOauthResponse>().await.unwrap();
        .text().await.unwrap();
    let res: GithubOauthResponse = serde_json::from_str(&res).unwrap();

    let user = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", res.access_token))
        .send().await.unwrap()
        .json::<GoogleUser>().await.unwrap();
    find_user_and_create_token(user.email, user.name, user.picture).await
}

pub async fn github_oauth(code: String) -> Result<(String, String), ApiResponse> {
    let form = reqwest::multipart::Form::new()
        .text("client_id", "Ov23liI2rrVysGbUPvxj")
        .text("client_secret", "e3f503ba66a0185c046eb7ec429b117d4acc7e94")
        .text("code", code);
    error!("{:?}", form);

    let res = reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .multipart(form)
        .header(reqwest::header::ACCEPT, "application/json")
        .send().await.unwrap()
        .text().await.unwrap();

    let res: GithubOauthResponse = serde_json::from_str(&res).unwrap();

    let user = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", res.access_token))
        .header(USER_AGENT, "blaze-backend-service")
        .send().await.unwrap()
        .json::<GithubUser>().await.unwrap();

    find_user_and_create_token(user.email, user.name, user.avatar_url).await
}

async fn find_user_and_create_token(email: String, name: String, image: String) -> Result<(String, String), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let existing_user: Option<User> = users::table
        .filter(users::email.eq(&email))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .await.optional()?;
    //
    let user = match existing_user {
        Some(user) => user,
        _ => {
            let user = User::new(name.to_string(), name.to_string().to_lowercase().replace(" ", "_") + &*Uuid::now_v7().to_string(), email, None, Some(image));
            diesel::insert_into(users::table)
                .values(user)
                .get_result(&mut conn)
                .await?
        }
    };
    create_token(user, &mut conn).await
}

async fn create_token(user: User, conn: &mut AsyncPgConnection) -> Result<(String, String), ApiResponse> {
    let token = match jwt_util::issue(user.id) {
        Ok(token) => token,
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };
    let jti = Uuid::now_v7();
    let refresh_token = match jwt_util::issue_refresh_token(user.id, jti) {
        Ok(token) => token,
        _ => return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
    };
    let _ = diesel::insert_into(user_session::table)
        .values(UserSession::new(jti, user.id))
        .get_result::<UserSession>(conn)
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