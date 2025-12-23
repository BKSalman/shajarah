use dioxus::{logger::tracing::Level, prelude::*};

use modules::add_request::AddMember;
use modules::admin::pages::{Admin, login::AdminLogin, register::AdminRegister};
use pages::Home;
use serde::{Deserialize, Serialize};

use crate::server::get_config;

pub mod config;
pub mod i18n;
#[cfg(feature = "server")]
pub mod middleware;
pub mod modules;
pub mod pages;
pub mod server;
pub mod ui;
pub mod util;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<Vec<String>>,
}

#[cfg(feature = "server")]
impl axum::response::IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        serde_json::to_string(&self)
            .expect("ErrorResponse as json")
            .into_response()
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Home,
    #[route("/admin")]
    Admin,
    #[route("/admin/login")]
    AdminLogin,
    #[route("/admin/register")]
    AdminRegister,
    #[route("/add")]
    AddMember,
}

#[cfg(feature = "server")]
async fn launch_server() -> Result<axum::Router, anyhow::Error> {
    use std::sync::Arc;

    use axum::{Extension, extract::DefaultBodyLimit, routing::get};
    use middleware::sessions::refresh_session;
    use modules::member::routes::export_members;
    use server::{AppState, InnerAppState};
    use sqlx::PgPool;
    use tower_cookies::CookieManagerLayer;
    use tower_http::{limit::RequestBodyLimitLayer, services::ServeDir};

    use crate::middleware::private_tree::block_non_invited;

    let pool =
        PgPool::connect(
                &std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL should be defined, example: postgres://postgres:shajarah-dev@localhost:5445/postgres")
            )
            .await
            .expect("Failed to connect to DB");

    let config = config::server::Config::load_config().unwrap();

    let (email_sender, email_receiver) = tokio::sync::mpsc::channel(10);

    let app_state = AppState(Arc::new(InnerAppState {
        db_pool: pool,
        email_sender,
        config,
    }));

    let cfg = ServeConfig::new();
    let mut router = axum::Router::new()
        .route("/api/v1/members/export", get(export_members))
        .with_state(app_state.clone())
        .serve_dioxus_application(cfg.clone(), app)
        .layer(axum::middleware::from_fn(block_non_invited))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
        .layer(Extension(app_state.clone()));

    if let Ok(dist) = std::env::var("SHAJARAH_DIST") {
        router = router.nest_service("/assets", ServeDir::new(dist));
    } else if let Some(dist) = option_env!("SHAJARAH_DIST") {
        router = router.nest_service("/assets", ServeDir::new(dist));
    } else {
        tracing::warn!("SHAJARAH_DIST is not set");
    }

    Ok(router)
}

fn main() {
    dioxus::logger::init(Level::INFO).ok();

    #[cfg(feature = "web")]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    {
        dioxus::serve(launch_server);
    }
}

fn app() -> Element {
    let config_resource = use_server_future(get_config)?.value();
    let config = config_resource.as_ref().unwrap().as_ref().unwrap().clone();
    use_context_provider(move || config);

    rsx! {
        document::Link { rel: "icon", href: asset!("/assets/favicon.ico") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/main.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/button.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/input.css") }
        document::Link {
            rel: "stylesheet",
            href: asset!("/src/ui/dropdown_menu/variants/main/style.css"),
        }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/form.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/card.css") }

        Router::<Route> {}
    }
}
