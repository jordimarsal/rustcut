use crate::shared::utils::create_api_key;
use crate::user::application::dtos::user_dto::{UserDto, UserDtoCreate, UserDtoCreateResponse};
use crate::user::domain::repositories::user_repository_port::UserRepositoryPort;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<dyn UserRepositoryPort + Send + Sync>,
}

impl UserService {
    pub fn new(user_repository: Arc<dyn UserRepositoryPort + Send + Sync>) -> Self {
        Self { user_repository }
    }

    pub async fn create_user(&self, user: UserDtoCreate) -> Result<UserDtoCreateResponse, Error> {
        let api_key = create_api_key();
        self.user_repository.create_user(user, api_key.clone()).await
    }

    pub async fn get_users(&self) -> Result<Vec<UserDto>, Error> {
        self.user_repository.get_users().await
    }

    pub async fn delete_user(&self, id: i32) -> Result<(), Error> {
        self.user_repository.delete_user(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::application::dtos::user_dto::{UserDtoCreate, UserDto};
    use async_trait::async_trait;
    use sqlx::Error;
    use std::sync::{Arc, Mutex};

    struct FakeUserRepo {
        users: Mutex<Vec<UserDto>>,
        next_id: Mutex<i64>,
    }

    impl FakeUserRepo {
        fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
                next_id: Mutex::new(1),
            }
        }
    }

    #[async_trait]
    impl crate::user::domain::repositories::user_repository_port::UserRepositoryPort for FakeUserRepo {
        async fn create_user(&self, user_dto: UserDtoCreate, _api_key: String) -> Result<crate::user::application::dtos::user_dto::UserDtoCreateResponse, Error> {
            let mut users = match self.users.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let mut id = match self.next_id.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let user = UserDto { id: *id, username: user_dto.username.clone(), email: user_dto.email.clone() };
            *id += 1;
            users.push(user);

            Ok(crate::user::application::dtos::user_dto::UserDtoCreateResponse { user: user_dto, api_key: String::from("fake-key") })
        }

        async fn get_users(&self) -> Result<Vec<UserDto>, Error> {
            let users = match self.users.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            Ok(users.clone())
        }

        async fn delete_user(&self, id: i32) -> Result<(), Error> {
            let mut users = match self.users.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            users.retain(|u| u.id != id as i64);
            Ok(())
        }
    }

    #[tokio::test]
    async fn user_service_create_get_delete() -> Result<(), Box<dyn std::error::Error>> {
        let repo = Arc::new(FakeUserRepo::new());
        let service = UserService::new(repo.clone());

        let dto = UserDtoCreate { username: "alice".into(), email: "alice@example.com".into() };
        let resp = service.create_user(dto.clone()).await?;

        assert_eq!(resp.user.username, dto.username);
        let users = service.get_users().await?;
        assert_eq!(users.len(), 1);
        let first = users.get(0).ok_or("expected one user but got none")?;
        assert_eq!(first.username, dto.username);

        service.delete_user(first.id as i32).await?;
        let users_after = service.get_users().await?;
        assert!(users_after.is_empty());

        Ok(())
    }
}
