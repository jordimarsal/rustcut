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
            email TEXT NOT NULL
        );
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
        ("JohnDoe", "john.doe@example.com"),
        ("JaneDoe", "jane.doe@example.com"),
        // Afegir més usuaris aquí
    ];

    for (username, email) in users {
        sqlx::query(
            r#"
            INSERT INTO users (username, email)
            SELECT ?1, ?2
            WHERE NOT EXISTS (
                SELECT 1 FROM users WHERE username = ?1 OR email = ?2
            );
            "#,
        )
        .bind(username)
        .bind(email)
        .execute(&**db_pool)
        .await?;
    }

    // sqlx::query("INSERT INTO users (username, email) VALUES (?, ?)")
    //     .bind("JohnDoe")
    //     .bind("john.doe@example.com")
    //     .execute(&**db_pool)
    //     .await?;

    Ok(())
}
