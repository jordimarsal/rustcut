use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Definim l'estructura URL
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct URL {
    pub key: String,
    pub secret_key: String,
    pub target_url: String,
    pub is_active: bool,
    pub clicks: i32,
    pub user_id: i32,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GeneratedKey {
    pub key_value: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct UsedKey {
    pub id: i32,
    pub key_value: String,
    pub user_id: i32,
}
