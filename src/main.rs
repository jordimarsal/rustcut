use actix_web::{web, App, HttpServer};

mod config;
mod user;

use crate::config::database::connect_to_db;
use crate::user::application::controllers::user_controller::{create_user, get_users};
use crate::user::domain::repositories::user_repository::UserRepository;
use std::sync::Arc;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Estableix la connexió a la base de dades
    let pool = connect_to_db().await.expect("Failed to connect to DB");
    let user_repository = Arc::new(UserRepository::new(pool).await);

    // Configura el servidor Actix-web
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(user_repository.clone()))
            .service(create_user)
            .service(get_users)
    })
    .bind("127.0.0.1:8083")?
    .run()
    .await
}
