use crate::user::application::dtos::user_dto::{UserDto, UserDtoCreate, UserDtoCreateResponse};
use crate::user::domain::models::user::User;
use crate::user::domain::repositories::user_repository_port::UserRepositoryPort;
use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;
use sqlx::Error;

pub struct SqlxUserRepository {
    db_pool: SqlitePool,
}

impl SqlxUserRepository {
    pub async fn new(db_pool: SqlitePool) -> Self {
        SqlxUserRepository { db_pool }
    }
}

#[async_trait]
impl UserRepositoryPort for SqlxUserRepository {
    async fn create_user(&self, user_dto: UserDtoCreate, api_key: String) -> Result<UserDtoCreateResponse, Error> {
        let _user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, api_key) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&user_dto.username)
        .bind(&user_dto.email)
        .bind(api_key.clone())
        .fetch_one(&self.db_pool)
        .await?;

        Ok(UserDtoCreateResponse { user: user_dto, api_key })
    }

    async fn get_users(&self) -> Result<Vec<UserDto>, Error> {
        log::info!("Getting users");
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.db_pool)
            .await?;

        Ok(users
            .into_iter()
            .map(|user| UserDto {
                id: user.id,
                username: user.username,
                email: user.email,
            })
            .collect())
    }

    async fn delete_user(&self, id: i32) -> Result<(), Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}
