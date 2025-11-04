pub mod types;
use crate::{
    ErrorResponse,
    server::{AppState, InnerAppState},
};
use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tower_cookies::Cookies;
use uuid::Uuid;

pub const SESSION_COOKIE_NAME: &str = "session_id";

pub struct UserSession {
    pub session_id: Option<Uuid>,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error("Something went wrong")]
    SomethingWentWrong,
    #[error("Something went wrong")]
    Sqlx(#[from] sqlx::Error),
    #[error("Invalid session")]
    InvalidSession,
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self:#?}");
        match self {
            SessionError::SomethingWentWrong => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            SessionError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            SessionError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
        }
    }
}

impl FromRequestParts<AppState> for UserSession {
    type Rejection = SessionError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies =
            parts
                .extract::<Cookies>()
                .await
                .map_err(|(_error_status, error_message)| {
                    tracing::error!(
                        "session-extractor: failed to get private cookie jar: {error_message}"
                    );
                    SessionError::SomethingWentWrong
                })?;
        if let Some(session_id) = cookies
            .private(&state.0.config.cookies_secret)
            .get(SESSION_COOKIE_NAME)
        {
            Ok(Self {
                session_id: Some(Uuid::parse_str(session_id.value()).map_err(|e| {
                    tracing::error!("session-extractor: invalid session_id: {e}");
                    SessionError::InvalidSession
                })?),
            })
        } else {
            Ok(Self { session_id: None })
        }
    }
}

pub async fn refresh_session(
    session: UserSession,
    State(state): State<Arc<InnerAppState>>,
    request: Request,
    next: Next,
) -> Result<Response, SessionError> {
    tracing::info!("running refresh_session middleware");
    if let Some(session_id) = session.session_id {
        sqlx::query!(
            r#"
                UPDATE sessions
                SET expires_at = $1
                WHERE id = $2 AND expires_at = $3
            "#,
            Utc::now() + Duration::days(2),
            session_id,
            Utc::now(),
        )
        .execute(&state.db_pool)
        .await?;
    }

    Ok(next.run(request).await)
}
