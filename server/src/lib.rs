use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::FromRef,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod api;
pub mod auth;
pub mod pages;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<Vec<String>>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self)
            .expect("ErrorResponse as json")
            .into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}

#[derive(Serialize, Deserialize)]
pub struct EmailCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub credentials: EmailCredentials,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub cookie_secret: String,
    pub totp_encryption_key: Vec<u8>,
    pub email_config: Option<EmailConfig>,
    pub base_url: url::Url,
}

impl Config {
    pub fn load_config() -> Result<Self, ConfigError> {
        log::info!("getting config file");
        let config_path = std::env::var("SHAJARAH_CONFIG_PATH").unwrap_or_default();

        let mut config_path = PathBuf::from(config_path);

        config_path.set_file_name("config.toml");

        let config_file = std::fs::read_to_string(config_path)?;
        Ok(toml::from_str::<Config>(&config_file)?)
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "gender")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
}

impl core::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
        }
    }
}

impl PartialEq<str> for Gender {
    fn eq(&self, other: &str) -> bool {
        (matches!(self, Gender::Male) && other == "male")
            || (matches!(self, Gender::Female) && other == "female")
    }
}

impl<'a> PartialEq<&'a str> for Gender {
    fn eq(&self, other: &&'a str) -> bool {
        (matches!(self, Gender::Male) && *other == "male")
            || (matches!(self, Gender::Female) && *other == "female")
    }
}

pub type NodeId = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    id: NodeId,
    name: String,
    mother_name: String,
    father_name: String,
    children: Vec<Node>,
}

impl Node {
    pub fn new(
        id: NodeId,
        name: String,
        father_name: String,
        mother_name: String,
        children: Vec<Node>,
    ) -> Self {
        Self {
            id,
            name,
            children,
            mother_name,
            father_name,
        }
    }
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug)]
pub struct EmailMessage {
    pub to: String,
    pub content: String,
}

pub struct InnerAppState {
    pub db_pool: PgPool,
    pub cookies_secret: tower_cookies::Key,
    pub totp_encryption_key: aes_gcm::Key<aes_gcm::Aes256Gcm>,
    pub base_url: url::Url,
    pub email_sender: tokio::sync::mpsc::Sender<EmailMessage>,
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub inner: Arc<InnerAppState>,
}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
