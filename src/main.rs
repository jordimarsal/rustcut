use actix_web::{web, App};
#[cfg(not(test))]
use actix_web::HttpServer;

#[cfg(not(test))]
use log::info;
#[cfg(not(test))]
use log4rs;

mod config;
mod shared;
mod url;
mod user;

#[cfg(not(test))]
use crate::config::database::connect_to_db;
#[cfg(not(test))]
use crate::config::env::AppConfig;
use crate::url::application::controllers::url_controller::{
    create_url, delete_url, forward_to_target_url, get_url_info,
};
#[cfg(not(test))]
use crate::url::infra::sqlx_url_repository::SqlxURLRepository;
#[cfg(not(test))]
use crate::url::domain::repositories::url_repository_port::URLRepositoryPort;
#[cfg(not(test))]
use crate::url::domain::services::url_service::URLService;
use crate::user::application::controllers::user_controller::{create_user, delete_user, get_users};
#[cfg(not(test))]
use crate::user::infra::sqlx_user_repository::SqlxUserRepository;
#[cfg(not(test))]
use crate::user::domain::repositories::user_repository_port::UserRepositoryPort;
#[cfg(not(test))]
use crate::user::domain::services::user_service::UserService;

use std::sync::Arc;

pub fn configure_services(
    cfg: &mut web::ServiceConfig,
    user_service: crate::user::domain::services::user_service::UserService,
    url_service: crate::url::domain::services::url_service::URLService,
    app_config: crate::config::env::AppConfig,
) {
    cfg
        .app_data(web::Data::new(Arc::new(user_service.clone())))
        .app_data(web::Data::new(Arc::new(url_service.clone())))
        .app_data(app_config)
        .service(create_user)
        .service(get_users)
        .service(delete_user)
        .service(create_url)
        .service(forward_to_target_url)
        .service(get_url_info)
        .service(delete_url);
}

#[cfg(not(test))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicialitza el sistema de logs — si falla, inicialitzem un logger fallback i fem `log::error!`
    if let Err(err) = log4rs::init_file("log4rs.yml", Default::default()) {
        // intentar inicialitzar `env_logger` com a fallback (no fallarem si ja hi ha un logger)
        let _ = env_logger::try_init();
        log::error!("failed to initialize log4rs from log4rs.yml — using fallback logger: {}", err);
    }

    // Carrega les variables d'entorn i els arguments de la línia de comandes
    let config = AppConfig::from_env_and_args();
    let server_port = config.server_port.clone();
    let base_url = config.base_url.clone();
    let protocol = config.protocol.clone();

    // Estableix la connexió a la base de dades — propaguem l'error amb `?` i registrem detalls
    let pool = match connect_to_db().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to connect to DB: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "database connection failed"));
        }
    };

    let user_repository: Arc<dyn UserRepositoryPort + Send + Sync> = Arc::new(
        SqlxUserRepository::new(pool.clone()).await,
    );

    // Crear una nova instància de UserService amb UserRepositoryPort
    let user_service = UserService::new(user_repository.clone());

    let url_repository: Arc<dyn URLRepositoryPort + Send + Sync> = Arc::new(
        SqlxURLRepository::new(pool.clone()).await,
    );
    let url_service = URLService::new(url_repository.clone());

    info!("Server up in {protocol}://{base_url}:{server_port}");

    // Configura el servidor Actix-web (separa la configuració a `configure_services` per facilitar tests)
    HttpServer::new(move || {
        App::new().configure(|cfg| configure_services(cfg, user_service.clone(), url_service.clone(), config.clone()))
    })
    .bind(format!("{base_url}:{server_port}"))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{init_service, call_service, TestRequest};
    use std::sync::Arc;
    use async_trait::async_trait;
    use sqlx::Error;

    // Minimal fake UserRepo to construct a UserService for the test
    struct FakeUserRepo;
    impl FakeUserRepo { fn new() -> Self { Self } }

    #[async_trait]
    impl crate::user::domain::repositories::user_repository_port::UserRepositoryPort for FakeUserRepo {
        async fn create_user(&self, user_dto: crate::user::application::dtos::user_dto::UserDtoCreate, _api_key: String) -> Result<crate::user::application::dtos::user_dto::UserDtoCreateResponse, Error> {
            Ok(crate::user::application::dtos::user_dto::UserDtoCreateResponse { user: user_dto, api_key: "fake-key".into() })
        }
        async fn get_users(&self) -> Result<Vec<crate::user::application::dtos::user_dto::UserDto>, Error> { Ok(vec![]) }
        async fn delete_user(&self, _id: i32) -> Result<(), Error> { Ok(()) }
    }

    // Minimal fake URLRepo to construct a URLService for the test
    struct FakeURLRepo;
    #[async_trait]
    impl crate::url::domain::repositories::url_repository_port::URLRepositoryPort for FakeURLRepo {
        async fn create_url(&self, target_url: String, user_id: i32) -> Result<crate::url::domain::models::schema::URL, sqlx::Error> {
            Ok(crate::url::domain::models::schema::URL { key: "k".into(), secret_key: "s".into(), target_url, is_active: true, clicks: 0, user_id })
        }
        async fn get_db_url_by_key(&self, _url_key: String) -> Result<crate::url::domain::models::schema::URL, sqlx::Error> { Err(sqlx::Error::RowNotFound) }
        async fn get_db_url_by_user_and_target_url(&self, _user_id: i32, _target_url: String) -> Result<crate::url::domain::models::schema::URL, sqlx::Error> { Err(sqlx::Error::RowNotFound) }
        async fn get_user_by_apy_key(&self, _api_key: String) -> Result<i32, ()> { Ok(1) }
        async fn increment_clicks(&self, _url_key: String) -> sqlx::Result<()> { Ok(()) }
    }

    #[actix_web::test]
    async fn configure_services_registers_routes() {
        let user_repo = Arc::new(FakeUserRepo::new());
        let user_service = crate::user::domain::services::user_service::UserService::new(user_repo);
        let url_repo = Arc::new(FakeURLRepo);
        let url_service = crate::url::domain::services::url_service::URLService::new(url_repo);

        let cfg = crate::config::env::AppConfig::from_env_and_args();
        let app = init_service(App::new().configure(|c| configure_services(c, user_service.clone(), url_service.clone(), cfg.clone()))).await;

        // Call a registered route to ensure wiring ran
        let req = TestRequest::get().uri("/users").to_request();
        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}

