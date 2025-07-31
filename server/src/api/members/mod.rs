use axum::{http::StatusCode, response::IntoResponse};

use crate::{auth::AuthError, ErrorResponse};

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum MembersError {
    #[error("something went wrong")]
    SomethingWentWrong,

    #[error("bad request")]
    BadRequest,

    #[error("something went wrong")]
    Sqlx(#[from] sqlx::Error),

    #[error("no family members")]
    NoMembers,

    #[error("no root member")]
    NoRootMember,

    #[error("invalid {0} value")]
    InvalidValue(String),

    #[error("invalid field name: {0}")]
    InvalidField(String),

    #[error("invalid image type")]
    InvalidImage,

    #[error(transparent)]
    AuthError(#[from] AuthError),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Garde(#[from] garde::Report),

    #[error(transparent)]
    AES(#[from] aes_gcm::Error),

    #[error("{0}")]
    TOTPQr(String),

    #[error(transparent)]
    TOTPURL(#[from] totp_rs::TotpUrlError),
}

impl IntoResponse for MembersError {
    fn into_response(self) -> axum::response::Response {
        log::error!("{self:#?}");

        match self {
            MembersError::SomethingWentWrong => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            MembersError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            MembersError::AuthError(e) => e.into_response(),
            MembersError::NoMembers => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: self.to_string(),
                    details: None,
                },
            )
                .into_response(),
            MembersError::NoRootMember => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: self.to_string(),
                    details: None,
                },
            )
                .into_response(),
            MembersError::InvalidValue(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    details: None,
                },
            )
                .into_response(),
            MembersError::InvalidImage => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    details: None,
                },
            )
                .into_response(),
            MembersError::InvalidField(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: self.to_string(),
                    details: None,
                },
            )
                .into_response(),
            MembersError::BadRequest => (StatusCode::BAD_REQUEST).into_response(),
            MembersError::Anyhow(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: self.to_string(),
                    details: None,
                },
            )
                .into_response(),
            MembersError::Garde(_) => (StatusCode::BAD_REQUEST).into_response(),
            MembersError::AES(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            MembersError::TOTPQr(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            MembersError::TOTPURL(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
