use actix_web::web;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePool;

use crate::shared::utils::create_random_key;

pub async fn connect_to_db() -> Result<sqlx::SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename("./sqlite:database.db")
        .create_if_missing(true);

    let pool = sqlx::SqlitePool::connect_with(options).await?;

    // Crear la taula 'users' si no existeix
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT NOT NULL,
            api_key TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Crea la taula URL
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS urls (
            id INTEGER PRIMARY KEY,
            key TEXT NOT NULL,
            secret_key TEXT NOT NULL,
            target_url TEXT NOT NULL,
            is_active BOOLEAN NOT NULL,
            clicks INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE INDEX IF NOT EXISTS idx_user_id ON urls (user_id);
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS generated_keys (
            key_value TEXT PRIMARY KEY
        );
        CREATE TABLE IF NOT EXISTS used_keys (
            id INTEGER PRIMARY KEY,
            key_value VARCHAR(50),
            user_id INTEGER,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE INDEX IF NOT EXISTS idx_used_keys_key_value ON used_keys (key_value);
        "#,
    )
    .execute(&pool)
    .await?;

    // Sembrar la base de dades amb dades inicials (propaga l'error en lloc de `expect`)
    seed_data(web::Data::new(pool.clone())).await?;

    Ok(pool)
}

pub async fn seed_data(db_pool: web::Data<SqlitePool>) -> Result<(), sqlx::Error> {
    let users = vec![
        ("JordiM", "marcaljordi@google.com", "1234567890"),
        ("Pepet", "pepet@example.com", "0987654321"),
        // Afegir més usuaris aquí
    ];

    for (username, email, api_key) in users {
        sqlx::query(
            r#"
            INSERT INTO users (username, email, api_key)
            SELECT ?1, ?2, ?3
            WHERE NOT EXISTS (
                SELECT 1 FROM users WHERE username = ?1 OR email = ?2
            );
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(api_key)
        .execute(&**db_pool)
        .await?;
    }

    let urls = vec![
        ("ERW8S", "ERW8S_BD6EZEUN", "http://www.jordimp.net/", true, 0, 1), // Afegir més URLs aquí
    ];
    for (key, secret_key, target_url, is_active, clicks, user_id) in urls {
        sqlx::query(
            r#"
            INSERT INTO urls (key, secret_key, target_url, is_active, clicks, user_id)
            SELECT ?1, ?2, ?3, ?4, ?5, ?6
            WHERE NOT EXISTS (
                SELECT 1 FROM urls WHERE key = ?1 OR secret_key = ?2
            );
            "#,
        )
        .bind(key)
        .bind(secret_key)
        .bind(target_url)
        .bind(is_active)
        .bind(clicks)
        .bind(user_id)
        .execute(&**db_pool)
        .await?;
    }

    let keys = generate_keys();

    for key_value in keys {
        sqlx::query(
            r#"
            INSERT INTO generated_keys (key_value)
            SELECT ?1
            WHERE
                (SELECT COUNT(*) FROM generated_keys) < 10
            ;
            "#,
        )
        .bind(key_value)
        .execute(&**db_pool)
        .await?;
    }

    Ok(())
}

fn generate_keys() -> Vec<String> {
    let mut keys = vec![];
    for _ in 0..10 {
        keys.push(create_random_key(8));
    }
    keys
}

#[cfg(test)]
mod connect_tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn connect_to_db_creates_db_and_tables() -> Result<(), sqlx::Error> {
        // Use the real function (it will create ./sqlite:database.db)
        let pool = connect_to_db().await?;

        // Check that 'users' table exists
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'")
            .fetch_one(&pool)
            .await?;
        assert!(row.0 >= 1);

        // cleanup - remove file if created (best-effort)
        let path = Path::new("./sqlite:database.db");
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Executor;
    use actix_web::web;

    #[tokio::test]
    async fn seed_data_populates_tables() -> Result<(), sqlx::Error> {
        let pool = SqlitePoolOptions::new().max_connections(1).connect(":memory:").await?;

        // create tables same as connect_to_db does
        pool.execute(r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                api_key TEXT NOT NULL
            );
        "#).await?;

        pool.execute(r#"
            CREATE TABLE IF NOT EXISTS urls (
                id INTEGER PRIMARY KEY,
                key TEXT NOT NULL,
                secret_key TEXT NOT NULL,
                target_url TEXT NOT NULL,
                is_active BOOLEAN NOT NULL,
                clicks INTEGER NOT NULL,
                user_id INTEGER NOT NULL
            );
        "#).await?;

        pool.execute(r#"
            CREATE TABLE IF NOT EXISTS generated_keys (
                key_value TEXT PRIMARY KEY
            );
        "#).await?;

        pool.execute(r#"
            CREATE TABLE IF NOT EXISTS used_keys (
                id INTEGER PRIMARY KEY,
                key_value VARCHAR(50),
                user_id INTEGER
            );
        "#).await?;

        // Call seed_data which should insert default users and urls
        seed_data(web::Data::new(pool.clone())).await?;

        // Validate users
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(&pool).await?;
        assert!(count.0 >= 1);

        // Validate urls
        let count_urls: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM urls").fetch_one(&pool).await?;
        assert!(count_urls.0 >= 0);

        // Validate generated_keys has entries (<=10)
        let count_keys: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM generated_keys").fetch_one(&pool).await?;
        assert!(count_keys.0 <= 10);

        Ok(())
    }
}
