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
use crate::url::domain::repositories::url_repository::URLRepository;
use crate::url::domain::services::url_service::URLService;
use crate::user::application::controllers::user_controller::{create_user, delete_user, get_users};
use crate::user::domain::repositories::user_repository::UserRepository;
use crate::user::domain::services::user_service::UserService;

use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicialitza el sistema de logs
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Carrega les variables d'entorn i els arguments de la línia de comandes
    let config = AppConfig::from_env_and_args();
    let server_port = config.server_port.clone();
    let base_url = config.base_url.clone();
    let protocol = config.protocol.clone();

    // Estableix la connexió a la base de dades
    let pool = connect_to_db().await.expect("Failed to connect to DB");

    let user_repository = Arc::new(UserRepository::new(pool.clone()).await);

    // Crear una nova instància de UserService amb UserRepository
    let user_service = UserService::new(user_repository.clone());

    let url_repository = Arc::new(URLRepository::new(pool.clone(), config.clone()).await);
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
