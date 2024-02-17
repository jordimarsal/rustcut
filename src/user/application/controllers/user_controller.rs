use actix_web::{delete, get, post, web, HttpResponse, Responder};

use crate::user::application::dtos::user_dto::UserDtoCreate;
use crate::user::domain::services::user_service::UserService;

use std::sync::Arc;

#[post("/users")]
pub async fn create_user(
    user_dto: web::Json<UserDtoCreate>, user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    match user_service.create_user(user_dto.into_inner()).await {
        Ok(user_dto_response) => HttpResponse::Ok().json(user_dto_response),
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

#[delete("/users/{id}")]
async fn delete_user(user_service: web::Data<Arc<UserService>>, id: web::Path<i32>) -> impl Responder {
    match user_service.delete_user(id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().body("User deleted successfully"),
        Err(_) => HttpResponse::InternalServerError().body("Error deleting user"),
    }
}
