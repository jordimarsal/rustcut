use crate::url::application::dtos::url_dto::{CustomError, URLBaseDto, URLInfoDto};
use crate::url::domain::repositories::url_repository::URLRepository;

use log::debug;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct URLService {
    url_repository: Arc<URLRepository>,
}

impl URLService {
    pub fn new(url_repository: Arc<URLRepository>) -> Self {
        Self { url_repository }
    }

    pub async fn create_url(&self, url_base: URLBaseDto) -> Result<URLInfoDto, Error> {
        debug!("Creating URL");
        let user_id = self.url_repository.get_user_by_apy_key(url_base.api_key.clone()).await
        .map_err(|_| CustomError::new(400, "No valid API_KEY"))?;
        debug!("User id: {}", user_id);
        self.url_repository.create_url(url_base.target_url, user_id).await
    }

    pub async fn forward_to_target_url(&self, url_key: String) -> Result<String, Error> {
        let target_url = self.url_repository.get_db_url_by_key(url_key).await?;
        let _ = self.url_repository.increment_clicks(target_url.clone()).await?;
        Ok(target_url)
    }

    pub async fn get_url_info(&self, url_key: String) -> Result<String, Error> {
        self.url_repository.get_db_url_by_key(url_key).await
    }

    pub async fn delete_url(&self, url_key: String) -> Result<String, Error> {
        self.url_repository.get_db_url_by_key(url_key).await
    }
}
