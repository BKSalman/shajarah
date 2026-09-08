use axum::{
    extract::{Request, State},
    http::{Method, header::ACCEPT},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{modules::admin::server::has_admin, server::InnerAppState};

const REGISTER_PATH: &str = "/admin/register";
const LOGIN_PATH: &str = "/admin/login";

/// An admin is only ever created once, so cache the answer and stop hitting the
/// database on every page load once we've seen one.
static ADMIN_EXISTS: AtomicBool = AtomicBool::new(false);

/// Until the first admin registers there is nothing to show, so every page load
/// lands on the registration page. Once an admin exists that page is gone.
pub async fn gate_admin_register(
    State(state): State<Arc<InnerAppState>>,
    request: Request,
    next: Next,
) -> Response {
    // only page loads, never assets or server functions
    let is_page_load = request.method() == Method::GET
        && request
            .headers()
            .get(ACCEPT)
            .and_then(|accept| accept.to_str().ok())
            .is_some_and(|accept| accept.contains("text/html"));

    if is_page_load {
        let on_register_page = request.uri().path() == REGISTER_PATH;

        match admin_exists(&state.db_pool).await {
            Ok(false) if !on_register_page => return Redirect::to(REGISTER_PATH).into_response(),
            Ok(true) if on_register_page => return Redirect::to(LOGIN_PATH).into_response(),
            Ok(_) => {}
            Err(e) => tracing::error!("failed to check for an existing admin: {e}"),
        }
    }

    next.run(request).await
}

async fn admin_exists(db_pool: &sqlx::PgPool) -> Result<bool, sqlx::Error> {
    if ADMIN_EXISTS.load(Ordering::Relaxed) {
        return Ok(true);
    }

    let exists = has_admin(db_pool).await?;

    if exists {
        ADMIN_EXISTS.store(true, Ordering::Relaxed);
    }

    Ok(exists)
}
