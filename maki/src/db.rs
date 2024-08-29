use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use tracing::info;

pub async fn get_pool() -> anyhow::Result<PgPool, anyhow::Error> {
    info!(
            target: "db",
        "Connecting to the database at url {}",
        &env::var("DATABASE_URL")?
    );
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .min_connections(1)
        .max_lifetime(std::time::Duration::from_secs(10))
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    info!(target: "db", "Connected to the database!");

    // run migrations
    info!(target: "db", "Running migrations...");
    sqlx::migrate!().run(&pool).await?;

    info!(target: "db", "Migrations complete!");
    Ok(pool)
}
