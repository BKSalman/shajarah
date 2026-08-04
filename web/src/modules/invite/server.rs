use dioxus::prelude::*;
use uuid::Uuid;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::AuthExtractor;
    pub use crate::modules::invite::types::InviteRow;
    pub use crate::modules::user::types::UserRole;
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
    pub use jiff::tz::TimeZone;
    pub use sha2::Digest as _;
    pub use sha2::Sha256;
}
use crate::modules::invite::types::Invite;

#[cfg(feature = "server")]
use server_imports::*;

#[post("/api/v1/invites", admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn create_invite(expires_at: Option<jiff::Zoned>) -> anyhow::Result<Uuid> {
    let token = Uuid::new_v4();
    let mut hasher = Sha256::default();
    hasher.update(token.to_string().as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    sqlx::query!(
        r#"
            INSERT INTO user_invites (token_hash, created_by, expires_at)
            VALUES ($1, $2, $3::text::timestamptz);
        "#,
        token_hash,
        admin.current_user.id,
        expires_at.map(|z| z.timestamp().to_string())
    )
    .execute(&state.db_pool)
    .await?;

    Ok(token)
}

#[get("/api/v1/invites", _admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn get_invites() -> anyhow::Result<Vec<Invite>> {
    let invites = sqlx::query_as!(
        InviteRow,
        r#"
            SELECT
                id,
                token_hash,
                created_by,
                expires_at as "expires_at: jiff_sqlx::Timestamp",
                used_at as "used_at: jiff_sqlx::Timestamp",
                accepted_by,
                created_at as "created_at: jiff_sqlx::Timestamp"
            FROM user_invites
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?
    .into_iter()
    .map(|r| Invite {
        id: r.id,
        token_hash: r.token_hash,
        created_by: r.created_by,
        expires_at: r.expires_at.map(|t| t.to_jiff().to_zoned(TimeZone::UTC)),
        used_at: r.used_at.map(|t| t.to_jiff().to_zoned(TimeZone::UTC)),
        accepted_by: r.accepted_by,
        created_at: r.created_at.to_jiff().to_zoned(TimeZone::UTC),
    })
    .collect();

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
