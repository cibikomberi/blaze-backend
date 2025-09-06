use bb8::{Pool, PooledConnection, RunError};
use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, PoolError};
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use tokio::sync::OnceCell;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
static POOL: OnceCell<DbPool> = OnceCell::const_new();

pub async fn init() {
    info!("Running migrations");
    let connection_url = env::var("DATABASE_URL").unwrap();
    let mut connection = PgConnection::establish(&connection_url).expect("Error connecting to database");
    connection.run_pending_migrations(MIGRATIONS).expect("Error running migrations");
    info!("Migrations complete");

    info!("Initializing connection pool");
    let _ = get_connection_pool().await;
}

async fn build_connection_pool() -> DbPool {
    let connection_url = env::var("DATABASE_URL").unwrap();
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(connection_url);
    Pool::builder().build(manager).await.unwrap()
}

pub async fn get_connection_pool() -> &'static DbPool {
    POOL.get_or_init(build_connection_pool).await
}

pub async fn get_connection() -> Result<PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>, RunError<PoolError>> {
    let pool = get_connection_pool().await;
    pool.get().await
}