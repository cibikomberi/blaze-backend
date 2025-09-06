use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::{Error, HttpMessage};
use actix_web::middleware::Next;
use crate::error::ApiResponse;
use crate::util::jwt_util::decode;
use crate::user::user_service;

pub async fn jwt_auth (request: ServiceRequest, next: Next<impl MessageBody>) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth_header = request.headers().get("Authorization");
    if let None = auth_header {

    };
    let auth_header = auth_header.unwrap().to_str().unwrap();
    let sub = match decode(auth_header) {
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
        Ok(token_data) => token_data.claims.sub,
    };
    debug!("JWT auth: {:?}", sub);

    let user = user_service::find_by_id(sub)
        .await.ok_or(ApiResponse::new(StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    request.extensions_mut().insert(user);

    next.call(request).await
}