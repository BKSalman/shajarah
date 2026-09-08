use dioxus::{logger::tracing::Level, prelude::*};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::user::pages::login::UserLogin;
use crate::modules::user::pages::register::UserRegister;
use components::loading::FullPageLoading;
use modules::add_request::AddMemberRequest;
use modules::admin::pages::{
    Admin, AdminLayout, invites::AdminInvites, login::AdminLogin, register::AdminRegister,
    settings::AdminSettings,
};
use modules::settings::server::get_settings;
use pages::{Home, HomeLayout};
use server::get_config;

pub mod components;
pub mod config;
pub mod i18n;
#[cfg(feature = "server")]
pub mod middleware;
pub mod modules;
pub mod pages;
pub mod server;
pub mod ui;
pub mod util;

#[used]
static FONT_AWESOME: Asset = asset!(
    "/assets/fontawesome",
    AssetOptions::folder().with_hash_suffix(false)
);

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

#[rustfmt::skip]
#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/admin/login")]
    AdminLogin,
    #[route("/admin/register")]
    AdminRegister,
    #[layout(AdminLayout)]
        #[route("/admin")]
        Admin,
        #[route("/admin/settings")]
        AdminSettings,
        #[route("/admin/invites")]
        AdminInvites,
    #[end_layout]
    #[layout(HomeLayout)]
        #[layout(PrivateTreeGuard)]
            #[route("/")]
            Home,
        #[end_layout]
        #[route("/add")]
        AddMemberRequest,
    #[end_layout]
    #[route("/register?:invite_token")]
    UserRegister { invite_token: Uuid },
    #[route("/login")]
    UserLogin,
    #[route("/unauthorized")]
    Unauthorized,
}

#[component]
pub fn PrivateTreeGuard() -> Element {
    let config = use_context::<config::client::Config>();
    let is_authed = use_server_future(crate::modules::user::server::get_user)?;

    use_effect(move || {
        if !config.public && matches!(is_authed(), Some(Err(_))) {
            navigator().replace(Route::Unauthorized {});
        }
    });

    if config.public {
        return rsx! {
            Outlet::<Route> {
            }
        };
    }

    match is_authed() {
        None => rsx! {
            FullPageLoading {
            }
        },
        Some(Err(_)) => rsx! {}, // effect handles the redirect
        Some(Ok(_)) => rsx! {
            Outlet::<Route> {
            }
        },
    }
}

#[cfg(feature = "server")]
async fn launch_server(config: config::server::Config) -> Result<axum::Router, anyhow::Error> {
    use std::sync::Arc;

    use axum::{Extension, extract::DefaultBodyLimit, routing::get};
    use middleware::admin_gate::gate_admin_register;
    use middleware::sessions::refresh_session;
    use middleware::settings_gate::gate_add_page;
    use modules::member::routes::export_members;
    use server::{AppState, InnerAppState};
    use sqlx::PgPool;
    use tower_cookies::CookieManagerLayer;
    use tower_http::{
        compression::CompressionLayer, limit::RequestBodyLimitLayer, services::ServeDir,
    };

    use crate::middleware::private_tree::block_non_invited;

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL should be defined, example: postgres://postgres:shajarah-dev@localhost:5432/postgres");
    tracing::debug!("connecting to {}", db_url);

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!().run(&pool).await?;

    let (email_sender, email_receiver) = tokio::sync::mpsc::channel(10);

    let app_state = AppState(Arc::new(InnerAppState {
        db_pool: pool,
        email_sender,
        config,
    }));

    let cfg = ServeConfig::new();
    let mut router = dioxus::server::router(app)
        .route("/api/v1/members/export", get(export_members))
        // .with_state(app_state.clone())
        // .layer(axum::middleware::from_fn(block_non_invited))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            gate_add_page,
        ))
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            gate_admin_register,
        ))
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
        .layer(CompressionLayer::new())
        .layer(Extension(app_state.clone()));

    // if let Ok(dist) = std::env::var("SHAJARAH_DIST") {
    //     router = router.nest_service("/assets", ServeDir::new(dist));
    // } else if let Some(dist) = option_env!("SHAJARAH_DIST") {
    //     router = router.nest_service("/assets", ServeDir::new(dist));
    // } else {
    //     tracing::warn!("SHAJARAH_DIST is not set");
    // }

    Ok(router)
}

fn main() {
    dioxus::logger::init(Level::DEBUG).ok();

    #[cfg(feature = "web")]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    {
        let config = config::server::Config::load_config().unwrap();

        dioxus::serve(move || launch_server(config.clone()));
    }
}

#[component]
fn Unauthorized() -> Element {
    rsx! {
        div { "Unauthorized" }
    }
}

fn app() -> Element {
    let config_resource = use_server_future(get_config)?.value();
    let config = config_resource.as_ref().unwrap().as_ref().unwrap().clone();
    use_context_provider(move || config);

    let settings_resource = use_server_future(get_settings)?.value();
    let settings = settings_resource
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    use_context_provider(move || settings);

    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
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
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/loading.css") }
        document::Link {
            rel: "stylesheet",
            href: format!("{FONT_AWESOME}/css/all.min.css"),
        }

        Router::<Route> {
        }
    }
}
