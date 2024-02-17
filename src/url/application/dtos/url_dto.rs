use serde::{Deserialize, Serialize};

// Definim l'estructura URL que hereta de URLBase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct URLBaseDto {
    pub target_url: String,
    pub api_key: String,
}

// Definim l'estructura URL que hereta de URLBase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct URLDto {
    pub target_url: String,
    pub is_active: bool,
    pub clicks: i32,
}

// Definim l'estructura URLInfo que hereta de URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct URLInfoDto {
    pub target_url: String,
    pub is_active: bool,
    pub clicks: i32,
    pub url: String,
    pub admin_url: String,
}
