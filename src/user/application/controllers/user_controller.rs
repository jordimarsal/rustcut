use actix_web::{get, post, web, HttpResponse, Responder};

use crate::user::application::dtos::user_dto::UserDto;
use crate::user::domain::repositories::user_repository::UserRepository;

#[post("/users")]
pub async fn create_user(user_dto: web::Json<UserDto>, user_repository: web::Data<UserRepository>) -> impl Responder {
    match user_repository.create_user(user_dto.into_inner()).await {
        Ok(user_dto) => HttpResponse::Ok().json(user_dto),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/users")]
pub async fn get_users(user_repository: web::Data<UserRepository>) -> impl Responder {
    match user_repository.get_users().await {
        Ok(users_dto) => HttpResponse::Ok().json(users_dto),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
