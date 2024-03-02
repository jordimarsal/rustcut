use actix_web::{delete, get, post, web, HttpResponse, Responder, http};

use crate::url::application::dtos::url_dto::URLBaseDto;
use crate::url::domain::services::url_service::URLService;

use log::debug;
use std::sync::Arc;

#[post("/url")]
pub async fn create_url(
    url_base_dto: web::Json<URLBaseDto>, url_service: web::Data<Arc<URLService>>,
) -> impl Responder {
    debug!("Creating URL");
    match url_service.create_url(url_base_dto.into_inner()).await {
        Ok(url_response) => HttpResponse::Ok().json(url_response),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

#[get("/{url_key}")]
pub async fn forward_to_target_url(url_key: web::Path<String>, url_service: web::Data<Arc<URLService>>) -> impl Responder {
    debug!("controller Forwarding to target URL: {}", url_key.clone());
    match url_service.forward_to_target_url(url_key.into_inner()).await {
        //Ok(target_url) => HttpResponse::Ok().json(target_url),
        Ok(target_url) => HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, target_url))
            .finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/admin/{secret_key}")]
pub async fn get_url_info(url_key: String, url_service: web::Data<Arc<URLService>>) -> impl Responder {
    debug!("Getting URL info");
    match url_service.get_url_info(url_key).await {
        Ok(users_dto) => HttpResponse::Ok().json(users_dto),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/admin/{secret_key}")]
pub async fn delete_url(url_key: String, url_service: web::Data<Arc<URLService>>) -> impl Responder {
    match url_service.delete_url(url_key).await {
        Ok(users_dto) => HttpResponse::Ok().json(users_dto),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
