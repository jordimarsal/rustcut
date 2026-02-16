use actix_web::{delete, get, post, web, HttpResponse, Responder, http};

use crate::config::env::AppConfig;
use crate::url::application::dtos::url_dto::URLBaseDto;
use crate::url::application::mappers::mappers::map_url_to_dto;
use crate::url::domain::services::url_service::URLService;

use log::debug;
use std::sync::Arc;

#[post("/url")]
pub async fn create_url(
    url_base_dto: web::Json<URLBaseDto>, url_service: web::Data<Arc<URLService>>, config: web::Data<AppConfig>,
) -> impl Responder {
    debug!("Creating URL");
    match url_service.create_url(url_base_dto.into_inner()).await {
        Ok(url_model) => {
            let dto = map_url_to_dto(&url_model, config.get_ref().clone());
            HttpResponse::Ok().json(dto)
        }
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::App;
    use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
    use crate::config::env::AppConfig;
    use crate::url::domain::models::schema::URL;
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;
    use serde_json::Value;

    struct FakeRepo {
        url: Mutex<Option<URL>>,
        incremented: Mutex<bool>,
    }

    impl FakeRepo {
        fn new(u: Option<URL>) -> Self { Self { url: Mutex::new(u), incremented: Mutex::new(false) } }
    }

    #[async_trait]
    impl crate::url::domain::repositories::url_repository_port::URLRepositoryPort for FakeRepo {
        async fn create_url(&self, _target_url: String, _user_id: i32) -> Result<URL, sqlx::Error> {
            let mut guard = self.url.lock().unwrap();
            if let Some(u) = guard.clone() { Ok(u) } else { let new = URL{ key: "k".into(), secret_key: "s".into(), target_url: "http://t".into(), is_active: true, clicks: 0, user_id: 1 }; *guard = Some(new.clone()); Ok(new) }
        }
        async fn get_db_url_by_key(&self, _url_key: String) -> Result<URL, sqlx::Error> { self.url.lock().unwrap().clone().ok_or(sqlx::Error::RowNotFound) }
        async fn get_db_url_by_user_and_target_url(&self, _user_id: i32, _target_url: String) -> Result<URL, sqlx::Error> { self.url.lock().unwrap().clone().ok_or(sqlx::Error::RowNotFound) }
        async fn get_user_by_apy_key(&self, api_key: String) -> Result<i32, ()> { if api_key == "valid" { Ok(1) } else { Err(()) } }
        async fn increment_clicks(&self, _url_key: String) -> sqlx::Result<()> { *(self.incremented.lock().unwrap()) = true; Ok(()) }
    }

    #[actix_web::test]
    async fn controller_create_and_map_to_dto() {
        let repo = Arc::new(FakeRepo::new(None));
        let service = URLService::new(repo.clone());
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).app_data(web::Data::new(cfg.clone())).service(create_url)).await;

        let req = TestRequest::post().uri("/url").set_json(&URLBaseDto{ target_url: "http://x".into(), api_key: "valid".into() }).to_request();
        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = read_body_json(resp).await;
        assert!(body.get("url").is_some());
    }

    #[actix_web::test]
    async fn controller_forward_sets_location() {
        let url = URL{ key: "k".into(), secret_key: "s".into(), target_url: "http://target".into(), is_active: true, clicks: 0, user_id: 1 };
        let repo = Arc::new(FakeRepo::new(Some(url)));
        let service = URLService::new(repo.clone());
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).service(forward_to_target_url)).await;

        let req = TestRequest::get().uri("/k").to_request();
        let resp = call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::SEE_OTHER);
        let hdr = resp.headers().get(actix_web::http::header::LOCATION).unwrap().to_str().unwrap();
        assert!(hdr.contains("http://target"));
    }

    #[actix_web::test]
    async fn controller_get_url_info_returns_dto() {
        let url = URL{ key: "k".into(), secret_key: "s".into(), target_url: "http://target".into(), is_active: true, clicks: 2, user_id: 1 };
        let repo = Arc::new(FakeRepo::new(Some(url)));
        let service = URLService::new(repo.clone());
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).app_data(web::Data::new(cfg.clone())).service(get_url_info)).await;

        let req = TestRequest::get().uri("/admin/s").to_request();
        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = read_body_json(resp).await;
        assert_eq!(body.get("clicks").and_then(|v| v.as_i64()).unwrap_or(0), 2);
    }

    #[actix_web::test]
    async fn controller_create_returns_500_on_invalid_api_key() {
        let repo = Arc::new(FakeRepo::new(None));
        let service = URLService::new(repo.clone());
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).app_data(web::Data::new(cfg.clone())).service(create_url)).await;

        let req = TestRequest::post().uri("/url").set_json(&URLBaseDto{ target_url: "http://x".into(), api_key: "invalid".into() }).to_request();
        let resp = call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn controller_get_url_info_not_found_returns_500() {
        let repo = Arc::new(FakeRepo::new(None));
        let service = URLService::new(repo.clone());
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).app_data(web::Data::new(cfg.clone())).service(get_url_info)).await;

        let req = TestRequest::get().uri("/admin/unknown").to_request();
        let resp = call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    async fn controller_delete_url_returns_dto_on_success() {
        let url = URL{ key: "k".into(), secret_key: "s".into(), target_url: "http://target".into(), is_active: true, clicks: 0, user_id: 1 };
        let repo = Arc::new(FakeRepo::new(Some(url)));
        let service = URLService::new(repo.clone());
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).app_data(web::Data::new(cfg.clone())).service(delete_url)).await;

        let req = TestRequest::delete().uri("/admin/s").to_request();
        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Value = read_body_json(resp).await;
        assert!(body.get("admin_url").is_some());
    }

    #[actix_web::test]
    async fn controller_delete_url_not_found_returns_500() {
        let repo = Arc::new(FakeRepo::new(None));
        let service = URLService::new(repo.clone());
        let cfg = AppConfig { base_url: "localhost".into(), server_port: "8080".into(), protocol: "http".into() };
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).app_data(web::Data::new(cfg.clone())).service(delete_url)).await;

        let req = TestRequest::delete().uri("/admin/unknown").to_request();
        let resp = call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
}



#[get("/{url_key}")]
pub async fn forward_to_target_url(url_key: web::Path<String>, url_service: web::Data<Arc<URLService>>) -> impl Responder {
    debug!("controller Forwarding to target URL: {}", url_key.clone());
    match url_service.forward_to_target_url(url_key.into_inner()).await {
        Ok(target_url) => HttpResponse::SeeOther()
            .append_header((http::header::LOCATION, target_url))
            .finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/admin/{secret_key}")]
pub async fn get_url_info(url_key: String, url_service: web::Data<Arc<URLService>>, config: web::Data<AppConfig>) -> impl Responder {
    debug!("Getting URL info");
    match url_service.get_url_info(url_key).await {
        Ok(url_model) => {
            let dto = map_url_to_dto(&url_model, config.get_ref().clone());
            HttpResponse::Ok().json(dto)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/admin/{secret_key}")]
pub async fn delete_url(url_key: String, url_service: web::Data<Arc<URLService>>, config: web::Data<AppConfig>) -> impl Responder {
    match url_service.delete_url(url_key).await {
        Ok(url_model) => {
            let dto = map_url_to_dto(&url_model, config.get_ref().clone());
            HttpResponse::Ok().json(dto)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
