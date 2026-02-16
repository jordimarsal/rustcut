use async_trait::async_trait;
use crate::user::application::dtos::user_dto::{UserDto, UserDtoCreate, UserDtoCreateResponse};
use sqlx::Error;

#[async_trait]
pub trait UserRepositoryPort: Send + Sync {
    async fn create_user(&self, user_dto: UserDtoCreate, api_key: String) -> Result<UserDtoCreateResponse, Error>;
    async fn get_users(&self) -> Result<Vec<UserDto>, Error>;
    async fn delete_user(&self, id: i32) -> Result<(), Error>;
}
