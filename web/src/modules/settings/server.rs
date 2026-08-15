use dioxus::prelude::*;

use crate::modules::settings::types::Settings;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::AuthExtractor;
    pub use crate::modules::user::types::UserRole;
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
}

#[cfg(feature = "server")]
use server_imports::*;

#[get("/api/v1/settings", Extension(state): Extension<AppState>)]
pub async fn get_settings() -> anyhow::Result<Settings> {
    let settings = sqlx::query_as!(
        Settings,
        r#"
            SELECT add_page_enabled
            FROM settings
            WHERE id = TRUE
        "#,
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(settings)
}

#[put("/api/v1/settings", _admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn update_settings(settings: Settings) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            UPDATE settings
            SET add_page_enabled = $1, updated_at = now()
            WHERE id = TRUE
        "#,
        settings.add_page_enabled,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[cfg(feature = "server")]
pub async fn add_page_enabled(db_pool: &sqlx::PgPool) -> Result<bool, sqlx::Error> {
    let rec = sqlx::query!(
        r#"
            SELECT add_page_enabled
            FROM settings
            WHERE id = TRUE
        "#,
    )
    .fetch_one(db_pool)
    .await?;

    Ok(rec.add_page_enabled)
}
