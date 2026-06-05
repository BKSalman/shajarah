use dioxus::prelude::*;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::{AuthError, AuthExtractor};
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
}

#[cfg(feature = "server")]
use server_imports::*;

use crate::modules::user::types::{UserResponseBrief, UserRole};

#[get("/api/v1/user/admin", admin: Result<AuthExtractor<{ UserRole::Admin as u8 }>, AuthError>)]
pub async fn is_authorized() -> anyhow::Result<UserResponseBrief> {
    Ok(admin.or_unauthorized("Unauthorized")?.current_user)
}
