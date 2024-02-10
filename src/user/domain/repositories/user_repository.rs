use sqlx::sqlite::SqlitePool;
use crate::user::domain::models::user::User;
use crate::user::application::dtos::user_dto::UserDto;
use log::info;

pub struct UserRepository {
    db_pool: SqlitePool,
}

impl UserRepository {
    pub async fn new(db_pool: SqlitePool) -> Self {
        info!("Creating UserRepository");
        UserRepository { db_pool }
    }

    pub async fn create_user(&self, user_dto: UserDto) -> Result<UserDto, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *",
        )
        .bind(&user_dto.username)
        .bind(&user_dto.email)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(UserDto {
            id: user.id,
            username: user.username,
            email: user.email,
        })
    }

    pub async fn get_users(&self) -> Result<Vec<UserDto>, sqlx::Error> {
        log::info!("Getting users");
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.db_pool)
            .await?;

        Ok(users.into_iter().map(|user| UserDto {
            id: user.id,
            username: user.username,
            email: user.email,
        }).collect())
    }
}