use crate::config::env::AppConfig;
use crate::url::application::dtos::url_dto::{CustomError, URLInfoDto};
use crate::url::application::mappers::mappers::map_url_to_dto;
use crate::url::domain::models::schema::{GeneratedKey, URL};
use log::debug;
use sqlx::error::DatabaseError;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

pub struct URLRepository {
    db_pool: SqlitePool,
    config: AppConfig,
}

impl URLRepository {
    pub async fn new(db_pool: SqlitePool, config: AppConfig) -> Self {
        URLRepository { db_pool, config }
    }

    pub async fn create_url(&self, target_url: String, user_id: i32) -> Result<URLInfoDto, sqlx::Error> {
        debug!("Creating URL");
        let secret_key = &self.get_generated_key().await?;
        debug!("Secret key: {}", secret_key.clone());
        // secret_key es de la forma "key_1234"
        // key es la part de l'esquerra de la cadena
        let key = secret_key.split('_').collect::<Vec<&str>>()[0];
        let db_url = URL {
            target_url: target_url.clone(),
            key: key.to_string(),
            secret_key: secret_key.clone(),
            is_active: true,
            clicks: 0,
            user_id,
        };
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
        let res = map_url_to_dto(&db_url, self.config.clone());
        Ok(res)
    }

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

    pub async fn get_user_by_apy_key(&self, api_key: String) -> Result<i32, CustomError> {
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
        // .map_err(|err| {
        //     eprintln!("Error occurred[get_generated_key]: {}", err);
        //     err
        // })?;
        //.map_err(|error| CustomError::from_database_error(URLRepository::boxed_error(error.as_database_error().clone()), "No valid API_KEY"))?;
        .map_err(|error| CustomError::new(400, "No valid API_KEY"))?;


        Ok(result_api_key.get("id"))
    }

    // pub fn boxed_error(error: Option<&dyn DatabaseError>) -> Box<dyn DatabaseError> {
    //     match error {
    //         Some(error) => Box::new(error),
    //         None => Box::new(CustomError::new(0, "No error"))
    //     }
    // }

    // pub fn from_database_error(error: Option<&dyn DatabaseError>, message: &str) -> CustomError {
    //     let mut error_ref: &dyn DatabaseError = error.unwrap();
    //     let n_error: &mut dyn DatabaseError = &mut *error_ref;
    //     CustomError::DatabaseError {
    //         source: n_error,
    //         message: message.to_string(),
    //     }
    // }

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
