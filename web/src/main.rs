use dioxus::{logger::tracing::Level, prelude::*};

use modules::add_request::AddMember;
use modules::admin::pages::{Admin, login::AdminLogin, register::AdminRegister};
use pages::Home;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
pub mod config;
pub mod i18n;
#[cfg(feature = "server")]
pub mod middleware;
pub mod modules;
pub mod pages;
#[cfg(feature = "server")]
pub mod server;
pub mod ui;

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
async fn launch_server() {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };

    use dioxus_devtools::DevserverMsg;
    use tokio::net::TcpStream;
    use tokio_util::task::LocalPoolHandle;

    use axum::{extract::DefaultBodyLimit, routing::get};
    use dioxus::server::ServeConfigBuilder;
    use middleware::sessions::refresh_session;
    use modules::member::routes::export_members;
    use server::{AppState, InnerAppState};
    use sqlx::PgPool;
    use tower_cookies::CookieManagerLayer;
    use tower_http::limit::RequestBodyLimitLayer;

    let (devtools_tx, mut devtools_rx) = tokio::sync::mpsc::unbounded_channel();

    dioxus_devtools::connect(move |msg| _ = devtools_tx.send(msg));

    let pool =
        PgPool::connect(
                &std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL should be defined, example: postgres://postgres:shajarah-dev@localhost:5445/postgres")
            )
            .await
            .expect("Failed to connect to DB");

    let config = config::Config::load_config().unwrap();

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

    let router = axum::Router::<AppState>::new()
        .route("/api/members/export", get(export_members))
        .serve_dioxus_application(
            ServeConfigBuilder::new()
                .context(app_state.clone())
                .build()
                .unwrap(),
            App,
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
        .with_state(app_state.clone());

    let task_pool = LocalPoolHandle::new(
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
    );

    let mut make_service = router.into_make_service();

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    tracing::trace!("Listening on {address}");

    #[derive(Debug)]
    enum Msg {
        TcpStream(std::io::Result<(TcpStream, SocketAddr)>),
        Devtools(DevserverMsg),
    }

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(0);
    let mut hr_idx = 0;

    loop {
        let res = tokio::select! {
            res = listener.accept() => Msg::TcpStream(res),
            msg = devtools_rx.recv(), if !devtools_rx.is_closed() => {
                if let Some(msg) = msg {
                    Msg::Devtools(msg)
                } else {
                    continue;
                }
            }
        };

        match res {
            Msg::Devtools(devserver_msg) => match devserver_msg {
                DevserverMsg::HotReload(hot_reload_msg) => {
                    if hot_reload_msg.for_build_id == Some(dioxus::cli_config::build_id()) {
                        if let Some(table) = hot_reload_msg.jump_table {
                            use dioxus::server::{RenderHandleState, SSRState, render_handler};

                            unsafe { dioxus_devtools::subsecond::apply_patch(table).unwrap() };

                            let new_router = axum::Router::new().serve_static_assets();

                            let new_cfg = ServeConfigBuilder::new()
                                .context(app_state.clone())
                                .build()
                                .unwrap();

                            let hot_root = subsecond::HotFn::current(App);
                            let new_root_addr = hot_root.ptr_address().0 as usize as *const ();
                            let new_root = unsafe {
                                std::mem::transmute::<*const (), fn() -> Element>(new_root_addr)
                            };

                            let state = RenderHandleState::new(new_cfg.clone(), new_root)
                                .with_ssr_state(SSRState::new(&new_cfg));

                            let fallback_handler =
                                axum::routing::get(render_handler).with_state(state);

                            make_service = new_router
                                .fallback(fallback_handler)
                                .layer(axum::middleware::from_fn_with_state(
                                    app_state.clone(),
                                    refresh_session,
                                ))
                                .layer(CookieManagerLayer::new())
                                .layer(DefaultBodyLimit::disable())
                                .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
                                .with_state(app_state.clone())
                                .into_make_service();

                            shutdown_tx.send_modify(|i| {
                                *i += 1;
                                hr_idx += 1;
                            });
                        }
                    }
                }
                DevserverMsg::FullReloadStart => {}
                DevserverMsg::FullReloadFailed => {}
                DevserverMsg::FullReloadCommand => {}
                DevserverMsg::Shutdown => {}
                _ => {}
            },
            Msg::TcpStream(Ok((tcp_stream, _remote_addr))) => {
                let this_hr_index = hr_idx;
                let mut make_service = make_service.clone();
                let mut shutdown_rx = shutdown_rx.clone();
                task_pool.spawn_pinned(move || async move {
                    use axum::{body::Body, extract::Request};
                    use hyper::body::Incoming;
                    use hyper_util::server::conn::auto::Builder as HyperBuilder;
                    use hyper_util::{
                        rt::{TokioExecutor, TokioIo},
                        service::TowerToHyperService,
                    };
                    use tower::{Service as _, ServiceExt as _};

                    let tcp_stream = TokioIo::new(tcp_stream);

                    std::future::poll_fn(|cx| {
                        use axum::{extract::Request, routing::IntoMakeService};

                        <IntoMakeService<axum::Router> as tower::Service<Request>>::poll_ready(
                            &mut make_service,
                            cx,
                        )
                    })
                    .await
                    .unwrap_or_else(|err| match err {});

                    let tower_service = make_service
                        .call(())
                        .await
                        .unwrap_or_else(|err| match err {})
                        .map_request(|req: Request<Incoming>| req.map(Body::new));

                    // upgrades needed for websockets
                    let builder = HyperBuilder::new(TokioExecutor::new());
                    let connection = builder.serve_connection_with_upgrades(
                        tcp_stream,
                        TowerToHyperService::new(tower_service),
                    );

                    tokio::select! {
                        res = connection => {
                            if let Err(_err) = res {
                                // This error only appears when the client doesn't send a request and
                                // terminate the connection.
                                //
                                // If client sends one request then terminate connection whenever, it doesn't
                                // appear.
                            }
                        }
                        _res = shutdown_rx.wait_for(|i| *i == this_hr_index + 1) => {}
                    }
                });
            }
            Msg::TcpStream(Err(e)) => {
                tracing::error!("{e}");
            }
        }
    }
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
                launch_server().await;
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
