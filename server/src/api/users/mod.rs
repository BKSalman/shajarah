use axum::{http::StatusCode, response::IntoResponse};

use crate::ErrorResponse;

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum UsersError {
    #[error("something went wrong")]
    InternalServerError,

    #[error("something went wrong")]
    Sqlx(#[from] sqlx::Error),

    #[error("user not found")]
    UserNotFound,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("bad request")]
    BadRequest,

    #[error("user has no posts")]
    HasNoPosts,

    #[error("already logged in")]
    AlreadyLoggedIn,

    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    AES(#[from] aes_gcm::Error),

    #[error("{0}")]
    Conflict(String),

    #[error(transparent)]
    Garde(#[from] garde::Report),

    #[error("Invalid invitation")]
    InvalidInvitation,

    #[error("Invalid invitation")]
    InvitationAlreadyUsed,

    #[error(transparent)]
    TOTPURL(#[from] totp_rs::TotpUrlError),

    #[error("{0}")]
    TOTPQr(String),

    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error(transparent)]
    SecretParse(#[from] totp_rs::SecretParseError),
}

impl IntoResponse for UsersError {
    fn into_response(self) -> axum::response::Response {
        log::error!("{self:#?}");

        match self {
            UsersError::UserNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::HasNoPosts => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::BadRequest => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::Conflict(_) => (
                StatusCode::CONFLICT,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::Argon2(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::AES(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::AlreadyLoggedIn => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::Garde(_) => (StatusCode::BAD_REQUEST).into_response(),
            UsersError::InvalidInvitation => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::InvitationAlreadyUsed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            UsersError::TOTPURL(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::TOTPQr(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::SystemTime(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            UsersError::SecretParse(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
