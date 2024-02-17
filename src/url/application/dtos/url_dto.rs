use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use sqlx::error::{DatabaseError, ErrorKind};

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

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Database error: {message}")]
    DatabaseError {
        source: Box<dyn DatabaseError>,
        message: String,
    },
    #[error("Custom error: {code}, {message}")]
    OtherError { code: i32, message: String },
}

impl DatabaseError for CustomError {
    fn code(&self) -> Option<Cow<str>> {
        None
    }
    fn message(&self) -> &str {
        match self {
            CustomError::DatabaseError { source, message } => message,
            CustomError::OtherError { code, message } => message,
        }
    }
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync> {
        self
    }
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }

}

impl CustomError {
    pub fn new(code: i32, message: &str) -> Self {
        Self::OtherError {
            code,
            message: message.to_string(),
        }
    }
    pub fn from_database_error(error: Box<dyn DatabaseError>, message: &str) -> Self {
        Self::DatabaseError {
            source: error,
            message: message.to_string(),
        }
    }
}
