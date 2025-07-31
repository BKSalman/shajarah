use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::StatusCode, response::IntoResponse, RequestPartsExt};
use chrono::Utc;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    api::{
        sessions::{SessionError, UserSession},
        users::models::{UserResponseBrief, UserRole},
    },
    AppState, ErrorResponse,
};

// TODO: add generic for UserRole
// this will allow for role checking
pub struct AuthExtractor<const USER_ROLE: u8> {
    pub current_user: UserResponseBrief,
    pub session_id: Uuid,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("something went wrong")]
    SomethingWentWrong,

    #[error("something went wrong")]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid session")]
    InvalidSession,

    #[error("invalid session")]
    SessionError(#[from] SessionError),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        log::error!("{self:#?}");

        match self {
            AuthError::SomethingWentWrong => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AuthError::Sqlx(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AuthError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
            AuthError::SessionError(e) => e.into_response(),
        }
    }
}

#[async_trait]
impl<const USER_ROLE: u8> FromRequestParts<AppState> for AuthExtractor<USER_ROLE> {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let session_id = parts
            .extract_with_state::<UserSession, _>(state)
            .await?
            .session_id
            .ok_or_else(|| {
                log::error!("auth-extractor: missing session_id");
                AuthError::InvalidSession
            })?;

        // Safety: USER_ROLE is only provided by casting UserRole variants
        let role: UserRole = unsafe { std::mem::transmute(USER_ROLE) };

        #[derive(FromRow)]
        struct AuthRow {
            user_id: Uuid,
            session_id: Uuid,
            first_name: String,
            email: String,
            role: UserRole,
        }

        match role {
            UserRole::Admin => {
                let Some(rec) = sqlx::query_as!(
                    AuthRow,
                    r#"
                        SELECT users.id as user_id, sessions.id as session_id, users.first_name, users.email, users.role as "role: UserRole" FROM sessions
                        INNER JOIN users
                          ON sessions.user_id = users.id
                        WHERE sessions.id = $1 AND sessions.expires_at > $2 AND users.role = 'admin'
                    "#,
                    session_id,
                    Utc::now(),
                )
                .fetch_optional(&state.inner.db_pool)
                .await? else {
                sqlx::query!(r#"DELETE FROM sessions WHERE id = $1"#, session_id)
                    .execute(&state.inner.db_pool)
                    .await
                    .ok();
                return Err(AuthError::InvalidSession);
            };
                Ok(AuthExtractor {
                    current_user: UserResponseBrief {
                        id: rec.user_id,
                        first_name: rec.first_name,
                        email: rec.email,
                        role: rec.role,
                    },
                    session_id: rec.session_id,
                })
            }
            UserRole::User => {
                let Some(rec) = sqlx::query_as!(
                    AuthRow,
                    r#"
                        SELECT users.id as user_id, sessions.id as session_id, users.email, users.first_name, users.role as "role: UserRole" FROM sessions
                        INNER JOIN users
                          ON sessions.user_id = users.id
                        WHERE sessions.id = $1 AND sessions.expires_at > $2
                    "#,
                    session_id,
                    Utc::now(),
                )
                .fetch_optional(&state.inner.db_pool)
                .await? else {
                sqlx::query!(r#"DELETE FROM sessions WHERE id = $1"#, session_id)
                    .execute(&state.inner.db_pool)
                    .await
                    .ok();
                return Err(AuthError::InvalidSession);
            };
                Ok(AuthExtractor {
                    current_user: UserResponseBrief {
                        id: rec.user_id,
                        first_name: rec.first_name,
                        email: rec.email,
                        role: rec.role,
                    },
                    session_id: rec.session_id,
                })
            }
        }
    }
}
