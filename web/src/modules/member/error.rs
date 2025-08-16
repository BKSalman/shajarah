use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug)]
pub enum MembersError {
    #[error("Something went wrong")]
    InternalServerError,

    #[error("Something went wrong")]
    Sqlx(#[from] sqlx::Error),
}

impl IntoResponse for MembersError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self:?}");

        match self {
            MembersError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            MembersError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}
