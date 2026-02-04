use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub async fn init_db(path: &Path) -> anyhow::Result<SqlitePool> {
    // Ensure file exists (empty) if not
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::File::create(path)?;
    }

    let db_url = format!("sqlite://{}", path.to_string_lossy());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run migrations
    // Use ./migrations to be explicit relative to Cargo.toml
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
