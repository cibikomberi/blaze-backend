#[macro_use]
extern crate log;
pub mod util;
pub mod schema;
mod error;
mod user;
mod auth;
mod organization;
mod bucket;
mod folder;
mod file;
mod config;

use crate::auth::auth_handler::auth_routes;
use crate::user::user_handler::user_routes;
use actix_files as fs;
use actix_web::dev::ServiceResponse;
use actix_web::http::{header, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers, Logger};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use env_logger::{init_from_env, Env};
use config::db_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    init_from_env(Env::default().default_filter_or("info"));
    db_config::init();
    info!("Starting http server: 127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(web::scope("/api")
                .service(web::scope("/user").configure(user_routes))
                .service(web::scope("/auth").configure(auth_routes)))
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
    HttpResponse::Ok().content_type("html").body(std::fs::read_to_string("./static/index.html").unwrap())
}