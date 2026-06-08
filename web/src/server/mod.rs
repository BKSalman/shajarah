use dioxus::prelude::*;

use crate::config;

#[cfg(feature = "server")]
mod server_only {
    pub use axum::Extension;
    use axum::extract::FromRef;
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
    pub struct AppState(pub Arc<InnerAppState>);

    impl std::ops::Deref for AppState {
        type Target = Arc<InnerAppState>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(feature = "server")]
pub use server_only::*;

#[get("/api/v1/config", state: Extension<AppState>)]
pub async fn get_config() -> anyhow::Result<config::client::Config> {
    let Extension(AppState(state)) = state;
    let config = config::client::Config {
        family_name: state.config.family_name.clone(),
        family_description: state.config.family_description.clone(),
        public: state.config.public,
    };

    Ok(config)
}
