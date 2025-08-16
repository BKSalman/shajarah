use std::sync::Arc;

use axum::extract::FromRef;
use dioxus::{prelude::*, server::server_context};
use sqlx::PgPool;

use crate::modules::member::types::{Gender, MemberRow};

#[derive(Debug)]
pub struct EmailMessage {
    pub to: String,
    pub content: String,
}

pub struct InnerAppState {
    pub db_pool: PgPool,
    pub cookies_secret: tower_cookies::Key,
    pub totp_encryption_key: aes_gcm::Key<aes_gcm::Aes256Gcm>,
    pub base_url: url::Url,
    pub email_sender: tokio::sync::mpsc::Sender<EmailMessage>,
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
