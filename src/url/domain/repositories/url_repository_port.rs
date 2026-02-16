use async_trait::async_trait;
use crate::url::domain::models::schema::URL;
use sqlx::Error;

#[async_trait]
pub trait URLRepositoryPort: Send + Sync {
    /// Create a new URL and return the domain `URL` model (mapping to DTOs happens in the application layer).
    async fn create_url(&self, target_url: String, user_id: i32) -> Result<URL, Error>;
    async fn get_db_url_by_key(&self, url_key: String) -> Result<URL, Error>;
    async fn get_db_url_by_user_and_target_url(&self, user_id: i32, target_url: String) -> Result<URL, Error>;
    async fn get_user_by_apy_key(&self, api_key: String) -> Result<i32, ()>;
    async fn increment_clicks(&self, url_key: String) -> sqlx::Result<()>;
}
