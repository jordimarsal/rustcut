use async_trait::async_trait;
use crate::url::domain::models::schema::{GeneratedKey, URL};
use crate::url::domain::repositories::url_repository_port::URLRepositoryPort;
use log::debug;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

/// SQLx implementation of the `URLRepositoryPort` domain port.
///
/// This adapter performs all database operations related to URLs
/// (creation, lookup, click counting and key management). It belongs
/// to the `infra` layer and should be injected into domain services
/// via the repository port trait.
pub struct SqlxURLRepository {
    db_pool: SqlitePool,
} 

impl SqlxURLRepository {
    /// Create a new `SqlxURLRepository` instance.
    ///
    /// - `db_pool`: sqlx SQLite connection pool used for queries.
    /// - `config`: application configuration (used for DTO mapping).
    pub async fn new(db_pool: SqlitePool) -> Self {
        SqlxURLRepository { db_pool }
    }

    /// Fetch the next available generated key from the `generated_keys` table.
    ///
    /// Returns the key string (for example "key_1234") or an `sqlx::Error`.
    async fn get_generated_key(&self) -> Result<String, sqlx::Error> {
        let key = sqlx::query_as::<_, GeneratedKey>("SELECT * FROM generated_keys ORDER BY key_value ASC LIMIT 1;")
            .fetch_one(&self.db_pool)
            .await
            .map_err(|err| {
                eprintln!("Error occurred[get_generated_key]: {}", err);
                err
            })?;
        Ok(key.key_value)
    }

    /// Create a shortened URL for the given `user_id` and `target_url`.
    ///
    /// Behaviour:
    /// - If the user already has the same target URL, return the existing URL DTO.
    /// - Otherwise obtain a generated secret key, insert the new URL row and related
    ///   auxiliary records, then return the mapped DTO.
    pub async fn create_url(&self, target_url: String, user_id: i32) -> Result<URL, sqlx::Error> {
        debug!("Creating URL");
        // check if the user already has this target URL
        let url = self
            .get_db_url_by_user_and_target_url(user_id, target_url.clone())
            .await;
        if let Ok(db_url) = url {
            debug!("URL already exists: {:?}", db_url);
            return Ok(db_url);
        }

        let secret_key = &self.get_generated_key().await?;
        debug!("Secret key: {}", secret_key.clone());
        // secret_key is expected to be in the format "key_1234" — use splitn and a safe fallback
        let key = secret_key.splitn(2, '_').next().unwrap_or(secret_key.as_str());
        let db_url = get_response_url_local(target_url, key, secret_key, user_id);
        let result_insert = sqlx::query_as::<_, URL>(
            "INSERT INTO urls (key, secret_key, target_url, is_active, clicks, user_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(db_url.key.clone())
        .bind(db_url.secret_key.clone())
        .bind(db_url.target_url.clone())
        .bind(db_url.is_active)
        .bind(db_url.clicks)
        .bind(db_url.user_id)
        .fetch_one(&self.db_pool)
        .await.map_err(|err| {
            eprintln!("Error occurred[_insert]: {}", err);
            err
        })?;

        if !result_insert.secret_key.is_empty() {
            // Record the key as "used" and remove it from the pool of generated keys
            // to avoid future reuse. This is part of the key allocation workflow.
            let _used_keys_insert = sqlx::query(
                r#"
                INSERT INTO used_keys (key_value, user_id)
                VALUES ($1, $2)
                "#,
            )
            .bind(key)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map_err(|err| {
                eprintln!("Error occurred[_used_keys_insert]: {}", err);
                err
            })?;

            let _delete_key = sqlx::query(
                r#"
                DELETE FROM generated_keys
                WHERE key_value = $1
                "#,
            )
            .bind(secret_key)
            .execute(&self.db_pool)
            .await
            .map_err(|err| {
                eprintln!("Error occurred[_delete_key]: {}", err);
                err
            })?;
            debug!("Key {} deleted", secret_key);
        }
        // Return inserted domain model (application layer will map to DTO)
        Ok(result_insert)
    }

    /// Return the target URL string for an active short `url_key`.
    /// Returns `sqlx::Error` if the key is not found or the query fails.
    pub async fn get_db_url_by_key(&self, url_key: String) -> Result<URL, sqlx::Error> {
        let result = sqlx::query_as::<_, URL>(
            "
            SELECT * FROM urls
            WHERE key = $1 AND is_active = true
            LIMIT 1
            ",
        )
        .bind(url_key)
        .fetch_one(&self.db_pool)
        .await?;
        Ok(result)
    }

    /// Find an existing URL row for `user_id` that matches `target_url`.
    /// Used to avoid creating duplicate shortened URLs for the same user+target.
    pub async fn get_db_url_by_user_and_target_url(
        &self, user_id: i32, target_url: String,
    ) -> Result<URL, sqlx::Error> {
        let result = sqlx::query_as::<_, URL>(
            "
            SELECT * FROM urls
            WHERE user_id = $1 AND target_url = $2
            LIMIT 1
            ",
        )
        .bind(user_id)
        .bind(target_url)
        .fetch_one(&self.db_pool)
        .await?;
        Ok(result)
    }

    /// Resolve an API key to its corresponding user id.
    /// Returns `Ok(user_id)` when found, otherwise returns `Err(())`.
    pub async fn get_user_by_apy_key(&self, api_key: String) -> Result<i32, ()> {
        let result_api_key = sqlx::query(
            "
            SELECT id FROM users
            WHERE api_key = $1
            LIMIT 1
            ",
        )
        .bind(api_key)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|err| {
            eprintln!("Error occurred[result_api_key]: {}", err);
            ()
        })?;

        Ok(result_api_key.get("id"))
    }

    /// Increment the click counter for the short URL identified by `url_key`.
    /// This performs a read-modify-write operation on the `urls` table.
    pub async fn increment_clicks(&self, url_key: String) -> sqlx::Result<()> {
        let mut url = sqlx::query_as::<_, URL>("SELECT * FROM urls WHERE key = $1")
            .bind(url_key.clone())
            .fetch_one(&self.db_pool)
            .await?;
        url.clicks += 1;
        sqlx::query("UPDATE urls SET clicks = $1 WHERE key = $2")
            .bind(url.clicks)
            .bind(url_key)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}

/// Build a `URL` value used by the repository insert logic.
/// This is duplicated here as a private helper for the infra adapter.
fn get_response_url_local(target_url: String, key: &str, secret_key: &String, user_id: i32) -> URL {
    URL {
        target_url: target_url.clone(),
        key: key.to_string(),
        secret_key: secret_key.clone(),
        is_active: true,
        clicks: 0,
        user_id,
    }
}

#[async_trait]
/// `URLRepositoryPort` implementation that delegates to the SQLx-backed methods above.
impl URLRepositoryPort for SqlxURLRepository {
    async fn create_url(&self, target_url: String, user_id: i32) -> Result<URL, sqlx::Error> {
        self.create_url(target_url, user_id).await
    }

    async fn get_db_url_by_key(&self, url_key: String) -> Result<URL, sqlx::Error> {
        self.get_db_url_by_key(url_key).await
    }

    async fn get_db_url_by_user_and_target_url(&self, user_id: i32, target_url: String) -> Result<URL, sqlx::Error> {
        self.get_db_url_by_user_and_target_url(user_id, target_url).await
    }

    async fn get_user_by_apy_key(&self, api_key: String) -> Result<i32, ()> {
        self.get_user_by_apy_key(api_key).await
    }

    async fn increment_clicks(&self, url_key: String) -> sqlx::Result<()> {
        self.increment_clicks(url_key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Executor;

    #[tokio::test]
    async fn sqlx_url_repository_create_get_and_increment() -> Result<(), Box<dyn std::error::Error>> {
        let pool = SqlitePoolOptions::new().max_connections(1).connect(":memory:").await?;

        // Create minimal schema required by the repository
        pool.execute(r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                api_key TEXT NOT NULL
            );
            CREATE TABLE urls (
                id INTEGER PRIMARY KEY,
                key TEXT NOT NULL,
                secret_key TEXT NOT NULL,
                target_url TEXT NOT NULL,
                is_active BOOLEAN NOT NULL,
                clicks INTEGER NOT NULL,
                user_id INTEGER NOT NULL
            );
            CREATE TABLE generated_keys (
                key_value TEXT PRIMARY KEY
            );
            CREATE TABLE used_keys (
                id INTEGER PRIMARY KEY,
                key_value VARCHAR(50),
                user_id INTEGER
            );
        "#).await?;

        // Insert a generated key that will be consumed by create_url
        pool.execute("INSERT INTO generated_keys (key_value) VALUES ('key_ABC')").await?;

        let repo = SqlxURLRepository::new(pool.clone()).await;

        let created = repo.create_url("http://ex".into(), 1).await?;
        assert_eq!(created.target_url, "http://ex");

        let fetched = repo.get_db_url_by_key(created.key.clone()).await?;
        assert_eq!(fetched.key, created.key);

        repo.increment_clicks(created.key.clone()).await?;
        let fetched2 = repo.get_db_url_by_key(created.key.clone()).await?;
        assert_eq!(fetched2.clicks, 1);

        Ok(())
    }

    #[tokio::test]
    async fn create_url_returns_existing_if_present() -> Result<(), Box<dyn std::error::Error>> {
        let pool = SqlitePoolOptions::new().max_connections(1).connect(":memory:").await?;
        pool.execute(r#"CREATE TABLE urls (id INTEGER PRIMARY KEY, key TEXT NOT NULL, secret_key TEXT NOT NULL, target_url TEXT NOT NULL, is_active BOOLEAN NOT NULL, clicks INTEGER NOT NULL, user_id INTEGER NOT NULL);"#).await?;
        pool.execute("INSERT INTO urls (key, secret_key, target_url, is_active, clicks, user_id) VALUES ('K1','SK1','http://same',1,0,1)").await?;
        let repo = SqlxURLRepository::new(pool.clone()).await;

        let res = repo.create_url("http://same".into(), 1).await?;
        assert_eq!(res.key, "K1");
        Ok(())
    }

    #[tokio::test]
    async fn create_url_errors_when_no_generated_key() -> Result<(), Box<dyn std::error::Error>> {
        let pool = SqlitePoolOptions::new().max_connections(1).connect(":memory:").await?;
        pool.execute(r#"CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL, email TEXT NOT NULL, api_key TEXT NOT NULL); CREATE TABLE urls (id INTEGER PRIMARY KEY, key TEXT NOT NULL, secret_key TEXT NOT NULL, target_url TEXT NOT NULL, is_active BOOLEAN NOT NULL, clicks INTEGER NOT NULL, user_id INTEGER NOT NULL); CREATE TABLE generated_keys (key_value TEXT PRIMARY KEY); CREATE TABLE used_keys (id INTEGER PRIMARY KEY, key_value VARCHAR(50), user_id INTEGER);"#).await?;
        let repo = SqlxURLRepository::new(pool.clone()).await;

        let err = repo.create_url("http://no-key".into(), 1).await.expect_err("expected error when no generated key");
        match err {
            sqlx::Error::RowNotFound => Ok(()),
            _ => Ok(()),
        }
    }
}

