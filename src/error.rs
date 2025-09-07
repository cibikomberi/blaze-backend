use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use diesel_async::pooled_connection::bb8::RunError;
use validator::ValidationError;

pub struct ApiResponse{
    status: StatusCode,
    message: String,
}

impl ApiResponse {
    // pub fn new(status: u16, message: String) -> ApiError{
    //     ApiError{ status: StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), message }
    // }
    // 
    pub fn new(status: StatusCode, message: String) -> ApiResponse{
        ApiResponse{ status, message }
    }
}

impl From<RunError>  for ApiResponse {
    fn from(e: RunError) -> ApiResponse{
        ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed getting db connection: {}", e))
    }
}

impl From<diesel::result::Error> for ApiResponse{
    fn from(e: diesel::result::Error) -> ApiResponse{
        ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed getting db connection: {}", e))
    }
}

impl std::fmt::Display for ApiResponse{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}
impl std::fmt::Debug for ApiResponse{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}
impl ResponseError for ApiResponse {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status)
            .set_body(BoxBody::new(self.message.to_string()))
    }
}

impl From<ValidationError> for ApiResponse{
    fn from(e: ValidationError) -> ApiResponse{
        ApiResponse::new(StatusCode::BAD_REQUEST, format!("Validation Error: {}", e))
    }
}