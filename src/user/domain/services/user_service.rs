use crate::user::application::dtos::user_dto::UserDto;
use crate::user::domain::repositories::user_repository::UserRepository;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn create_user(&self, user: UserDto) -> Result<UserDto, Error> {
        self.user_repository.create_user(user).await
    }

    pub async fn get_users(&self) -> Result<Vec<UserDto>, Error> {
        self.user_repository.get_users().await
    }
}
