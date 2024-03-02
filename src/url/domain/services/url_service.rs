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

    pub async fn create_url(&self, url_base: URLBaseDto) -> Result<URLInfoDto, CustomError> {
        debug!("Creating URL");
        let user_id = self
            .url_repository
            .get_user_by_apy_key(url_base.api_key.clone())
            .await
            .map_err(|_| CustomError::new(400, "No valid API_KEY"))?;
        debug!("User id: {}", user_id);
        let result = self
            .url_repository
            .create_url(url_base.target_url, user_id)
            .await
            .map_err(|err| {
                eprintln!("Error occurred[create_url_srvc]: {}", err);
                CustomError::new(500, "Error creating URL")
            });
        match result {
            Ok(url) => Ok(url),
            Err(e) => Err(e),
        }
    }

    pub async fn forward_to_target_url(&self, url_key: String) -> Result<String, Error> {
        let target_url = self.url_repository.get_db_url_by_key(url_key.clone()).await?;
        debug!("Forwarding to target URL: {}", target_url.clone());
        let _ = self.url_repository.increment_clicks(url_key.clone()).await?;
        Ok(target_url)
    }

    pub async fn get_url_info(&self, url_key: String) -> Result<String, Error> {
        self.url_repository.get_db_url_by_key(url_key).await
    }

    pub async fn delete_url(&self, url_key: String) -> Result<String, Error> {
        self.url_repository.get_db_url_by_key(url_key).await
    }
}
