use crate::{middleware::auth::AuthExtractor, modules::user::types::UserRole, server::AppState};
use axum::{
    RequestExt,
    extract::Query,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};

pub async fn block_non_invited(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let is_admin = request
        .extract_parts::<AuthExtractor<{ UserRole::Admin as u8 }>>()
        .await
        .is_ok();

    let AppState(app_state) = request.extensions().get::<AppState>().unwrap();

    if !app_state.config.public && !is_admin {
        let db_pool = app_state.db_pool.clone();
        #[derive(Deserialize, Serialize)]
        struct Invite {
            invite: uuid::Uuid,
        }

        let Ok(Query(invite)) = request.extract_parts::<Query<Invite>>().await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let Ok(invite_entry) =
            sqlx::query!("SELECT * FROM tree_invites WHERE id = $1", invite.invite)
                .fetch_optional(&db_pool)
                .await
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        if let Some(invite) = invite_entry {
            use chrono::Utc;

            if invite.expires_at < Utc::now() {
                return StatusCode::UNAUTHORIZED.into_response();
            }
        } else {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    let res = next.run(request).await;

    res
}
