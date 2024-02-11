use actix_web::{web, App, HttpServer};
use log::debug;
use log4rs;

mod config;
mod shared;
mod user;

use crate::config::database::connect_to_db;
use crate::user::application::controllers::user_controller::{create_user, get_users, delete_user};
use crate::user::domain::repositories::user_repository::UserRepository;
use crate::user::domain::services::user_service::UserService;

use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Inicialitza el sistema de logs
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    // Estableix la connexió a la base de dades
    let pool = connect_to_db().await.expect("Failed to connect to DB");

    let user_repository = Arc::new(UserRepository::new(pool).await);
    debug!("UserRepository created");

    // Crear una nova instància de UserService amb UserRepository
    let user_service = UserService::new(user_repository.clone());

    // Configura el servidor Actix-web
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::new(user_service.clone())))
            .service(create_user)
            .service(get_users)
            .service(delete_user)
    })
    .bind("127.0.0.1:8083")?
    .run()
    .await
}
