use dioxus::prelude::*;

use crate::config;

#[cfg(feature = "server")]
mod server_only {
    use axum::extract::FromRef;
    use dioxus::prelude::*;
    use sqlx::PgPool;
    use std::sync::Arc;

    use crate::config;

    #[derive(Debug)]
    pub struct EmailMessage {
        pub to: String,
        pub content: String,
    }

    pub struct InnerAppState {
        pub db_pool: PgPool,
        pub email_sender: tokio::sync::mpsc::Sender<EmailMessage>,
        pub config: config::server::Config,
    }

    #[derive(Clone, FromRef)]
    pub struct AppState {
        pub inner: Arc<InnerAppState>,
    }

    pub async fn get_state() -> ServerFnResult<Arc<InnerAppState>> {
        Ok(extract::<FromContext<AppState>, _>().await?.0.inner)
    }

    pub async fn get_cookies() -> ServerFnResult<tower_cookies::Cookies> {
        Ok(extract::<tower_cookies::Cookies, _>()
            .await
            .map_err(|_e| ServerFnError::new("Bad Request"))?)
    }
}

#[cfg(feature = "server")]
pub use server_only::*;

#[server]
pub async fn get_config() -> ServerFnResult<config::client::Config> {
    let state = get_state().await?;

    let config = config::client::Config {
        family_name: state.config.family_name.clone(),
    };

    Ok(config)
}
