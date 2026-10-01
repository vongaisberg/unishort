use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use rocket::http::Status;
use rocket::tokio::task;

use dotenv::dotenv;
use std::env;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

/// Initialize the database pool.
pub fn initialize(max_size: u32) -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create pool")
}

/// Runs `f` with a pooled connection on Tokio's blocking thread pool.
///
/// Diesel and r2d2 are synchronous: checking out a connection can wait for
/// the pool timeout, and every query blocks until Postgres answers. Doing
/// that on an async worker would stall every other request on that thread.
///
/// Fails with `ServiceUnavailable` when no connection can be checked out.
/// The closure's own result (usually a `QueryResult`) is passed through
/// untouched, so callers can still tell query errors apart.
pub async fn run<T, F>(pool: &PgPool, f: F) -> Result<T, Status>
where
    F: FnOnce(&PgConnection) -> T + Send + 'static,
    T: Send + 'static,
{
    let pool = pool.clone();
    task::spawn_blocking(move || match pool.get() {
        Ok(conn) => Ok(f(&conn)),
        Err(e) => {
            log::error!("Could not get a database connection: {}", e);
            Err(Status::ServiceUnavailable)
        }
    })
    .await
    .map_err(|e| {
        log::error!("Database task failed: {}", e);
        Status::InternalServerError
    })?
}
