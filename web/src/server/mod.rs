use std::sync::Arc;

use axum::extract::FromRef;
use dioxus::prelude::*;
use sqlx::PgPool;

pub struct InnerAppState {
    pub db_pool: PgPool,
    pub cookies_secret: tower_cookies::Key,
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub inner: Arc<InnerAppState>,
}

pub async fn get_state() -> ServerFnResult<Arc<InnerAppState>> {
    Ok(extract::<FromContext<AppState>, _>().await?.0.inner)
}

pub async fn get_cookies() -> ServerFnResult<tower_cookies::Cookies> {
    Ok(extract::<tower_cookies::Cookies, _>().await.map_err(|e| {
        tracing::error!("{e:?}");
        ServerFnError::new("Bad Request")
    })?)
}
