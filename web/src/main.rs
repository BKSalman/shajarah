use dioxus::{logger::tracing::Level, prelude::*};

use web::Route;

#[cfg(feature = "server")]
async fn launch_server(component: fn() -> Element) {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };

    use axum::{extract::DefaultBodyLimit, routing::get};
    use dioxus::server::ServeConfigBuilder;
    use sqlx::PgPool;
    use tower_cookies::CookieManagerLayer;
    use tower_http::limit::RequestBodyLimitLayer;
    use web::{
        middleware::sessions::refresh_session,
        modules::member::routes::export_members,
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
    let router = axum::Router::<AppState>::new()
        .route("/api/members/export", get(export_members))
        .serve_dioxus_application(
            ServeConfigBuilder::new()
                .context(app_state.clone())
                .build()
                .unwrap(),
            component,
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
        .with_state(app_state);

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
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
