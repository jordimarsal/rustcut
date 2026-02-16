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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::application::dtos::user_dto::UserDtoCreate;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Executor;

    #[tokio::test]
    async fn create_get_and_delete_user_integration() -> Result<(), Box<dyn std::error::Error>> {
        let pool = SqlitePoolOptions::new().max_connections(1).connect(":memory:").await?;

        pool.execute(r#"
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT NOT NULL,
                api_key TEXT NOT NULL
            );
        "#).await?;

        let repo = SqlxUserRepository::new(pool.clone()).await;
        let dto = UserDtoCreate { username: "bob".to_string(), email: "bob@example.com".to_string() };
        let resp = repo.create_user(dto.clone(), "apikey-integ".to_string()).await?;
        assert_eq!(resp.user.username, dto.username);

        let users = repo.get_users().await?;
        assert!(users.iter().any(|u| u.username == dto.username));

        let id = users.into_iter().find(|u| u.username == dto.username).unwrap().id as i32;
        repo.delete_user(id).await?;
        let users_after = repo.get_users().await?;
        assert!(users_after.iter().all(|u| u.id != id as i64));

        Ok(())
    }
}
