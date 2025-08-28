use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use diesel_async::pooled_connection::bb8::RunError;
use validator::ValidationError;

pub struct ApiError{
    status: StatusCode,
    message: String,
}

impl ApiError {
    // pub fn new(status: u16, message: String) -> ApiError{
    //     ApiError{ status: StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), message }
    // }
    // 
    pub fn new(status: StatusCode, message: String) -> ApiError{
        ApiError{ status, message }
    }
}

impl From<RunError>  for ApiError {
    fn from(e: RunError) -> ApiError{
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed getting db connection: {}", e))
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(value: diesel::result::Error) -> Self {
        error!("{}", value);
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "message".to_string())
    }
}

impl std::fmt::Display for ApiError{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}
impl std::fmt::Debug for ApiError{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status)
            .set_body(BoxBody::new(self.message.to_string()))
    }
}

impl From<ValidationError> for ApiError{
    fn from(e: ValidationError) -> ApiError{
        ApiError::new(StatusCode::BAD_REQUEST, format!("Validation Error: {}", e))
    }
}