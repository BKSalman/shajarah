use dioxus::prelude::*;
use uuid::Uuid;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::AuthExtractor;
    pub use crate::modules::user::types::UserRole;
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
    pub use sha2::Digest as _;
    pub use sha2::Sha256;
}
use crate::modules::invite::types::InviteRow;

#[cfg(feature = "server")]
use server_imports::*;

#[post("/api/v1/invites", admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn create_invite(
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> anyhow::Result<Uuid> {
    let token = Uuid::new_v4();
    let mut hasher = Sha256::default();
    hasher.update(token.to_string().as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    sqlx::query!(
        r#"
            INSERT INTO user_invites (token_hash, created_by, expires_at)
            VALUES ($1, $2, $3);
        "#,
        token_hash,
        admin.current_user.id,
        expires_at
    )
    .execute(&state.db_pool)
    .await?;

    Ok(token)
}

#[get("/api/v1/invites", _admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn get_invites() -> anyhow::Result<Vec<InviteRow>> {
    let invites = sqlx::query_as!(
        InviteRow,
        r#"
            SELECT * from user_invites
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(invites)
}

#[delete("/api/v1/invites/{id}", _admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn delete_invite(id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            DELETE FROM user_invites WHERE id = $1
        "#,
        id,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}
