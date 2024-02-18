use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Error, Debug, Serialize)]
pub struct CustomError {
    code: i32,
    message: String,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ 'code': {}, 'message': '{}' }}", self.code, self.message)
    }
}

impl CustomError {
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
}
