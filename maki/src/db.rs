use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub async fn get_pool() -> anyhow::Result<PgPool, anyhow::Error> {
    println!(
        "Connecting to the database at url {}",
        &env::var("DATABASE_URL")?
    );
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .min_connections(1)
        .max_lifetime(std::time::Duration::from_secs(10))
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    println!("Connected.");

    // run migrations
    println!("Running migrations...");
    sqlx::migrate!().run(&pool).await?;

    println!("Migrations are done!");
    Ok(pool)
}
