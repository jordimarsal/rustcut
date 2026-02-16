use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Definim l'estructura URL
#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct URL {
    pub key: String,
    pub secret_key: String,
    pub target_url: String,
    pub is_active: bool,
    pub clicks: i32,
    pub user_id: i32,
}

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct GeneratedKey {
    pub key_value: String,
}

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct UsedKey {
    pub id: i32,
    pub key_value: String,
    pub user_id: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_serde_roundtrip() {
        let url = URL { key: "k".into(), secret_key: "s".into(), target_url: "http://t".into(), is_active: true, clicks: 5, user_id: 1 };
        let j = serde_json::to_string(&url).expect("serialize");
        let back: URL = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back.key, url.key);
        assert_eq!(back.clicks, 5);
    }
}
