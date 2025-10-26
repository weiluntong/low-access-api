use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite};
use crate::models::User;
use anyhow::Result;

pub async fn init_db() -> Result<SqlitePool> {
    let database_url = "sqlite:./sso.db";
    
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        Sqlite::create_database(database_url).await?;
    }

    // Connect to database
    let pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &SqlitePool) -> Result<()> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'approved', 'denied'
            created_at TEXT NOT NULL,
            last_login TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create user_permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            permission TEXT NOT NULL,
            granted_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id),
            UNIQUE (user_id, permission)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn upsert_user(pool: &SqlitePool, user: &User) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, status, created_at, last_login)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            email = excluded.email,
            name = excluded.name,
            last_login = excluded.last_login
            -- Note: Don't update status on conflict, preserve admin decisions
        "#,
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.status)
    .bind(user.created_at.to_rfc3339())
    .bind(user.last_login.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}
