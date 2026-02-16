use actix_web::{web, App, HttpServer};
use log::info;
use log4rs;

mod config;
mod shared;
mod url;
mod user;

use crate::config::database::connect_to_db;
use crate::config::env::AppConfig;
use crate::url::application::controllers::url_controller::{
    create_url, delete_url, forward_to_target_url, get_url_info,
};
use crate::url::infra::sqlx_url_repository::SqlxURLRepository;
use crate::url::domain::repositories::url_repository_port::URLRepositoryPort;
use crate::url::domain::services::url_service::URLService;
use crate::user::application::controllers::user_controller::{create_user, delete_user, get_users};
use crate::user::infra::sqlx_user_repository::SqlxUserRepository;
use crate::user::domain::repositories::user_repository_port::UserRepositoryPort;
use crate::user::domain::services::user_service::UserService;

use std::sync::Arc;

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

    // Configura el servidor Actix-web
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::new(user_service.clone())))
            .app_data(web::Data::new(Arc::new(url_service.clone())))
            .app_data(config.clone())
            .service(create_user)
            .service(get_users)
            .service(delete_user)
            .service(create_url)
            .service(forward_to_target_url)
            .service(get_url_info)
            .service(delete_url)
    })
    .bind(format!("{base_url}:{server_port}"))?
    .run()
    .await
}
