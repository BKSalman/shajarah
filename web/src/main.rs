use dioxus::{logger::tracing::Level, prelude::*};

use web::Route;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const DIOXUS_COMPONENTS_THEME: Asset = asset!("/assets/styling/theme.css");
const DIOXUS_BUTTON_STYLES: Asset = asset!("/src/ui/button/variants/main/style.css");
const DIOXUS_INPUT_STYLES: Asset = asset!("/assets/styling/input.css");
const DIOXUS_DROPDOWN_STYLES: Asset = asset!("/src/ui/dropdown_menu/variants/main/style.css");

#[cfg(feature = "server")]
async fn launch_server(component: fn() -> Element) {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };

    use axum::extract::DefaultBodyLimit;
    use dioxus::server::ServeConfigBuilder;
    use sqlx::PgPool;
    use tower_cookies::CookieManagerLayer;
    use tower_http::limit::RequestBodyLimitLayer;
    use web::{
        middleware::sessions::refresh_session,
        server::{AppState, InnerAppState},
    };

    let pool =
        PgPool::connect(
                &std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL should be defined, example: postgres://postgres:shajarah-dev@localhost:5445/postgres")
            )
            .await
            .expect("Failed to connect to DB");

    let config = web::config::Config::load_config().unwrap();

    let (email_sender, email_receiver) = tokio::sync::mpsc::channel(10);

    let app_state = AppState {
        inner: Arc::new(InnerAppState {
            db_pool: pool,
            cookies_secret: tower_cookies::Key::from(&config.cookie_secret),
            totp_encryption_key: aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_exact_iter(
                config.totp_encryption_key,
            )
            .unwrap(),
            base_url: config.base_url,
            email_sender,
        }),
    };

    let ip =
        dioxus::cli_config::server_ip().unwrap_or_else(|| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = dioxus::cli_config::server_port().unwrap_or(8080);
    let address = SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let router = axum::Router::new()
        .serve_dioxus_application(
            ServeConfigBuilder::new()
                .context(app_state.clone())
                .build()
                .unwrap(),
            component,
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state,
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
        .into_make_service();
    axum::serve(listener, router).await.unwrap();
}

fn main() {
    dioxus::logger::init(Level::INFO).ok();

    #[cfg(feature = "web")]
    dioxus::launch(App);

    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                launch_server(App).await;
            });
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: DIOXUS_COMPONENTS_THEME }
        document::Link { rel: "stylesheet", href: DIOXUS_BUTTON_STYLES }
        document::Link { rel: "stylesheet", href: DIOXUS_INPUT_STYLES }
        document::Link { rel: "stylesheet", href: DIOXUS_DROPDOWN_STYLES }

        Router::<Route> {}
    }
}
