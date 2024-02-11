use actix_web::web;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePool;

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

    // Sembrar la base de dades amb dades inicials
    seed_data(web::Data::new(pool.clone()))
        .await
        .expect("Failed to seed data");

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
        ("ERW8S", "ERW8S_BD6EZEUN", "http://www.jordimp.net/", true, 0, 1)
        // Afegir més URLs aquí
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

    Ok(())
}
