use crate::{
    ErrorResponse,
    middleware::sessions::{SessionError, UserSession},
    modules::user::types::{UserResponseBrief, UserRole},
    server::AppState,
};
use axum::{RequestPartsExt, extract::FromRequestParts, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use dioxus::server::{FromContext, FromServerContext};
use sqlx::prelude::FromRow;
use uuid::Uuid;
#[derive(serde::Deserialize, serde::Serialize, Clone)]
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
        tracing::error!("{self:#?}");
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
                tracing::error!("auth-extractor: missing session_id");
                AuthError::InvalidSession
            })?;
        let role: UserRole = unsafe { std::mem::transmute(USER_ROLE) };
        #[derive(FromRow)]
        struct AuthRow {
            user_id: Uuid,
            session_id: Uuid,
            first_name: Option<String>,
            email: Option<String>,
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
                    session_id, Utc::now(),
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
                        first_name: rec.first_name.unwrap_or_default(),
                        email: rec.email.unwrap_or_default(),
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
                    session_id, Utc::now(),
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
                        first_name: rec.first_name.unwrap_or_default(),
                        email: rec.email.unwrap_or_default(),
                        role: rec.role,
                    },
                    session_id: rec.session_id,
                })
            }
        }
    }
}
#[async_trait::async_trait]
impl<const USER_ROLE: u8> FromServerContext<AppState> for AuthExtractor<USER_ROLE> {
    type Rejection = <AuthExtractor<USER_ROLE> as FromRequestParts<AppState>>::Rejection;
    async fn from_request(
        req: &dioxus::server::DioxusServerContext,
    ) -> Result<Self, Self::Rejection> {
        let state: FromContext<AppState> = req.extract().await.unwrap();
        let mut lock = req.request_parts_mut();
        AuthExtractor::<USER_ROLE>::from_request_parts(&mut lock, &state.0).await
    }
}
