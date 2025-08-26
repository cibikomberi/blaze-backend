use actix_web::{get, App, HttpServer};
use actix_files as fs;

#[get("/api")]
async fn hello() -> &'static str {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // .route("/", )
            .service(hello)
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
