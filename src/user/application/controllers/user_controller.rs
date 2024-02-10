use actix_web::{get, post, web, HttpResponse, Responder};

use crate::user::application::dtos::user_dto::UserDto;
use crate::user::domain::services::user_service::UserService;

use std::sync::Arc;

#[post("/users")]
pub async fn create_user(user_dto: web::Json<UserDto>, user_service: web::Data<Arc<UserService>>) -> impl Responder {
    match user_service.create_user(user_dto.into_inner()).await {
        Ok(user_dto) => HttpResponse::Ok().json(user_dto),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/users")]
pub async fn get_users(user_service: web::Data<Arc<UserService>>) -> impl Responder {
    match user_service.get_users().await {
        Ok(users_dto) => HttpResponse::Ok().json(users_dto),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
