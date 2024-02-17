use crate::url::application::dtos::url_dto::URLInfoDto;
use crate::url::application::mappers::mappers::map_url_to_dto;
use crate::url::domain::models::schema::{GeneratedKey, URL};
use sqlx::sqlite::SqlitePool;

pub struct URLRepository {
    db_pool: SqlitePool,
}

impl URLRepository {
    pub async fn new(db_pool: SqlitePool) -> Self {
        URLRepository { db_pool }
    }

    pub async fn create_url(&self, target_url: String, user_id: i32) -> Result<URLInfoDto, sqlx::Error> {
        let secret_key = &self.get_generated_key().await?;
        // secret_key es de la forma "key_1234"
        // key es la part de la dreta de la string
        let key = secret_key.split('_').collect::<Vec<&str>>()[1];
        let db_url = URL {
            target_url: target_url.clone(),
            key: key.to_string(),
            secret_key: secret_key.clone(),
            is_active: true,
            clicks: 0,
            user_id,
        };
        let _insert = sqlx::query_as::<_, URL>(
            "INSERT INTO urls (target_url, key, secret_key) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(db_url.target_url.clone())
        .bind(db_url.key.clone())
        .bind(db_url.secret_key.clone())
        .bind(db_url.is_active)
        .bind(db_url.clicks)
        .bind(db_url.user_id)
        .fetch_one(&self.db_pool)
        .await?;
        let res = map_url_to_dto(&db_url);
        Ok(res)
    }

    async fn get_generated_key(&self) -> Result<String, sqlx::Error> {
        let key = sqlx::query_as::<_, GeneratedKey>("SELECT * FROM generated_keys ORDER BY key_id ASC LIMIT 1;")
            .fetch_one(&self.db_pool)
            .await?;
        Ok(key.key_value)
    }

    pub async fn get_db_url_by_key(&self, url_key: String) -> Result<String, sqlx::Error> {
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
        Ok(result.target_url)
    }

    pub async fn increment_clicks(&self, url_key: String) -> sqlx::Result<()> {
        let mut url = sqlx::query_as::<_, URL>("SELECT * FROM urls WHERE key = $1")
            .bind(url_key.clone())
            .fetch_one(&self.db_pool)
            .await?;
        url.clicks += 1;
        sqlx::query("UPDATE urls SET clicks = $1 WHERE url_key = $2")
            .bind(url.clicks)
            .bind(url_key)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}
