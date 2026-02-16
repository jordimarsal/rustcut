use crate::url::application::dtos::url_dto::{CustomError, URLBaseDto};
use crate::url::domain::models::schema::URL;
use crate::url::domain::repositories::url_repository_port::URLRepositoryPort;

use log::debug;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct URLService {
    url_repository: Arc<dyn URLRepositoryPort + Send + Sync>,
}

impl URLService {
    pub fn new(url_repository: Arc<dyn URLRepositoryPort + Send + Sync>) -> Self {
        Self { url_repository }
    }

    /// Create a URL and return the domain `URL` model. Mapping to DTO is done in application layer.
    pub async fn create_url(&self, url_base: URLBaseDto) -> Result<URL, CustomError> {
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
        result
    }

    pub async fn forward_to_target_url(&self, url_key: String) -> Result<String, Error> {
        let url = self.url_repository.get_db_url_by_key(url_key.clone()).await?;
        let target_url = url.target_url.clone();
        debug!("Forwarding to target URL: {}", target_url.clone());
        let _ = self.url_repository.increment_clicks(url_key.clone()).await?;
        Ok(target_url)
    }

    pub async fn get_url_info(&self, url_key: String) -> Result<URL, Error> {
        self.url_repository.get_db_url_by_key(url_key).await
    }

    pub async fn delete_url(&self, url_key: String) -> Result<URL, Error> {
        self.url_repository.get_db_url_by_key(url_key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct FakeURLRepo {
        url_opt: Mutex<Option<URL>>,
        increment_called: Mutex<bool>,
        valid_api_key: Mutex<bool>,
    }

    impl FakeURLRepo {
        fn new(initial: Option<URL>) -> Self {
            Self { url_opt: Mutex::new(initial), increment_called: Mutex::new(false), valid_api_key: Mutex::new(true) }
        }
    }

    #[async_trait]
    impl crate::url::domain::repositories::url_repository_port::URLRepositoryPort for FakeURLRepo {
        async fn create_url(&self, _target_url: String, _user_id: i32) -> Result<URL, sqlx::Error> {
            let mut guard = self.url_opt.lock().unwrap();
            if let Some(u) = guard.clone() {
                Ok(u)
            } else {
                let new = URL { key: "k1".into(), secret_key: "s1".into(), target_url: "http://x".into(), is_active: true, clicks: 0, user_id: 1 };
                *guard = Some(new.clone());
                Ok(new)
            }
        }

        async fn get_db_url_by_key(&self, _url_key: String) -> Result<URL, sqlx::Error> {
            let guard = self.url_opt.lock().unwrap();
            guard.clone().ok_or_else(|| sqlx::Error::RowNotFound)
        }

        async fn get_db_url_by_user_and_target_url(&self, _user_id: i32, _target_url: String) -> Result<URL, sqlx::Error> {
            let guard = self.url_opt.lock().unwrap();
            guard.clone().ok_or_else(|| sqlx::Error::RowNotFound)
        }

        async fn get_user_by_apy_key(&self, api_key: String) -> Result<i32, ()> {
            let ok = *self.valid_api_key.lock().unwrap();
            if ok && api_key == "valid" { Ok(1) } else { Err(()) }
        }

        async fn increment_clicks(&self, _url_key: String) -> sqlx::Result<()> {
            let mut called = self.increment_called.lock().unwrap();
            *called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_url_returns_existing_or_new() {
        let existing = URL { key: "k-ex".into(), secret_key: "s-ex".into(), target_url: "http://ex".into(), is_active: true, clicks: 0, user_id: 1 };
        let repo = Arc::new(FakeURLRepo::new(Some(existing.clone())));
        let service = URLService::new(repo.clone());

        let dto = crate::url::application::dtos::url_dto::URLBaseDto { target_url: "http://ex".into(), api_key: "valid".into() };
        let res = service.create_url(dto).await.expect("should create/return");
        assert_eq!(res.key, existing.key);

        // Now use repo without existing URL
        let repo2 = Arc::new(FakeURLRepo::new(None));
        let service2 = URLService::new(repo2.clone());
        let dto2 = crate::url::application::dtos::url_dto::URLBaseDto { target_url: "http://new".into(), api_key: "valid".into() };
        let res2 = service2.create_url(dto2).await.expect("create new");
        assert_eq!(res2.key, "k1");
    }

    #[tokio::test]
    async fn create_url_invalid_api_key_returns_custom_error() {
        let repo = Arc::new(FakeURLRepo::new(None));
        let service = URLService::new(repo.clone());
        let dto = crate::url::application::dtos::url_dto::URLBaseDto { target_url: "http://x".into(), api_key: "invalid".into() };
        let res = service.create_url(dto).await;
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.to_string().contains("No valid API_KEY"), true);
    }

    #[tokio::test]
    async fn forward_to_target_url_increments_and_returns_target() {
        let url = URL { key: "k1".into(), secret_key: "s1".into(), target_url: "http://target".into(), is_active: true, clicks: 0, user_id: 1 };
        let repo = Arc::new(FakeURLRepo::new(Some(url.clone())));
        let service = URLService::new(repo.clone());

        let target = service.forward_to_target_url("k1".into()).await.expect("forward");
        assert_eq!(target, url.target_url);
        assert_eq!(*repo.increment_called.lock().unwrap(), true);
    }

    #[tokio::test]
    async fn get_info_and_delete_return_domain_model() {
        let url = URL { key: "k1".into(), secret_key: "s1".into(), target_url: "http://target".into(), is_active: true, clicks: 0, user_id: 1 };
        let repo = Arc::new(FakeURLRepo::new(Some(url.clone())));
        let service = URLService::new(repo.clone());

        let got = service.get_url_info("k1".into()).await.expect("get info");
        assert_eq!(got.key, url.key);

        let del = service.delete_url("k1".into()).await.expect("delete");
        assert_eq!(del.key, url.key);
    }
}
