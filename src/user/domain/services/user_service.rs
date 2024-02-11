use crate::user::application::dtos::user_dto::{UserDto, UserDtoCreateResponse, UserDtoCreate};
use crate::user::domain::repositories::user_repository::UserRepository;
use crate::shared::utils::create_api_key;
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
