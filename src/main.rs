#[macro_use]
extern crate log;
pub mod util;
pub mod schema;
mod error;
mod user;
mod auth;
mod organization;
mod bucket;
pub mod folder;
mod file;
mod config;

use crate::auth::auth_handler::auth_routes;
use crate::auth::auth_middleware::jwt_auth;
use crate::bucket::bucket_handler::bucket_routes;
use crate::file::file_handler::{file_routes, fs_routes};
use crate::folder::folder_handler::folder_routes;
use crate::organization::organization_handler::{organization_routes, sdk_routes};
use crate::user::user_handler::user_routes;
use actix_files as fs;
use actix_web::dev::ServiceResponse;
use actix_web::http::{header, StatusCode};
use actix_web::middleware::{from_fn, Compress, ErrorHandlerResponse, ErrorHandlers, Logger};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use config::db_config;
use env_logger::{init_from_env, Env};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_from_env(Env::default().default_filter_or("info"));
    db_config::init().await;
    info!("Starting http server: 127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .service(web::scope("/f").configure(fs_routes))
            .service(web::scope("/api")
                .service(web::scope("/user").configure(user_routes))
                .service(web::scope("/auth").configure(auth_routes))
                .service(web::scope("/organization").wrap(from_fn(jwt_auth)).configure(organization_routes))
                .service(web::scope("/bucket").wrap(from_fn(jwt_auth)).configure(bucket_routes))
                .service(web::scope("/folder").wrap(from_fn(jwt_auth)).configure(folder_routes))
                .service(web::scope("/file").wrap(from_fn(jwt_auth)).configure(file_routes)))
            .service(web::scope("/sdk").configure(sdk_routes))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
            .wrap(ErrorHandlers::new().handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header))
            .default_service(web::route().to(index))
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
fn add_error_header<B>(mut res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );

    // body is unchanged, map to "left" slot
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("html").body(tokio::fs::read_to_string("./static/index.html").await.unwrap())
}