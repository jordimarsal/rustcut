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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{init_service, call_service, TestRequest, read_body_json};
    use actix_web::App;
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;
    use crate::user::application::dtos::user_dto::{UserDtoCreate, UserDto, UserDtoCreateResponse};

    struct FakeUserRepo {
        users: Mutex<Vec<UserDto>>,
        next_id: Mutex<i64>,
    }

    impl FakeUserRepo {
        fn new() -> Self { Self { users: Mutex::new(Vec::new()), next_id: Mutex::new(1) } }
    }

    #[async_trait]
    impl crate::user::domain::repositories::user_repository_port::UserRepositoryPort for FakeUserRepo {
        async fn create_user(&self, user_dto: UserDtoCreate, api_key: String) -> Result<UserDtoCreateResponse, sqlx::Error> {
            let mut users = self.users.lock().unwrap();
            let mut id = self.next_id.lock().unwrap();
            let user = UserDto { id: *id, username: user_dto.username.clone(), email: user_dto.email.clone() };
            *id += 1;
            users.push(user);
            Ok(UserDtoCreateResponse { user: user_dto, api_key })
        }

        async fn get_users(&self) -> Result<Vec<UserDto>, sqlx::Error> {
            let users = self.users.lock().unwrap();
            Ok(users.clone())
        }

        async fn delete_user(&self, id: i32) -> Result<(), sqlx::Error> {
            let mut users = self.users.lock().unwrap();
            users.retain(|u| u.id != id as i64);
            Ok(())
        }
    }

    #[actix_web::test]
    async fn controller_create_get_delete_user() {
        let repo = Arc::new(FakeUserRepo::new());
        let service = crate::user::domain::services::user_service::UserService::new(repo.clone());
        let app = init_service(App::new().app_data(web::Data::new(Arc::new(service))).service(create_user).service(get_users).service(delete_user)).await;

        let dto = UserDtoCreate { username: "testuser".into(), email: "t@e.com".into() };
        let req = TestRequest::post().uri("/users").set_json(&dto).to_request();
        let resp = call_service(&app, req).await;
        assert!(resp.status().is_success());
        let created: UserDtoCreateResponse = read_body_json(resp).await;
        assert_eq!(created.user.username, "testuser");

        let req2 = TestRequest::get().uri("/users").to_request();
        let resp2 = call_service(&app, req2).await;
        let users: Vec<UserDto> = read_body_json(resp2).await;
        assert_eq!(users.len(), 1);

        let id = users[0].id;
        let req3 = TestRequest::delete().uri(&format!("/users/{}", id)).to_request();
        let resp3 = call_service(&app, req3).await;
        assert!(resp3.status().is_success());
    }
}
