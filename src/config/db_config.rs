use std::env;
use bb8::Pool;
use diesel_migrations::EmbeddedMigrations;
use actix_web::http::StatusCode;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::embed_migrations;
use crate::error::ApiError;
use tokio::sync::OnceCell;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
static POOL: OnceCell<DbPool> = OnceCell::const_new();

pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing DB...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    
    let pool: DbPool = Pool::builder()
        .build(manager)
        .await?;
    
    POOL.set(pool).map_err(|_| "Pool already initialized")?;
    
    // run_migrations().await?;
    info!("DB initialized successfully");
    Ok(())
}

// pub async  fn connection() -> Result<, ApiError> {
//     POOL.get_or_init().await.get()?
// }

async fn build_connection_pool() -> Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
    let connection_url = env::var("DATABASE_URL").unwrap();
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(connection_url);
    Pool::builder().build(manager).await.unwrap()
}

pub async fn get_connection_pool() -> &'static Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
    static POOL: OnceCell<Pool<AsyncDieselConnectionManager<AsyncPgConnection>>> = OnceCell::const_new();
    POOL.get_or_init(build_connection_pool).await
}