use std::sync::Arc;

use aes_gcm::KeyInit;
use axum::{
    extract::DefaultBodyLimit,
    handler::HandlerWithoutStateExt,
    http::{HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use rand::Rng;
use server::{
    api::{
        members::routes::{
            add_member, approve_member_request, delete_member, disapprove_member_request,
            edit_member, export_members, get_member_invites, get_members, get_members_flat,
            invite_member, request_add_member, upload_members_csv,
        },
        sessions::refresh_session,
        users::routes::{
            accept_member_invite, admin_login, decline_member_invite, logout, me, member_login,
            verify_totp,
        },
    },
    pages::{
        add_request_page, admin_login_page, admin_page, admin_register_page, invite_reply_page,
        members_login_page, user_page, NotFoundTemplate,
    },
    AppState, Config, ConfigError, EmailMessage, InnerAppState,
};

use server::api::users::routes::create_user;

use clap::Parser;
use sqlx::PgPool;
use std::net::{Ipv4Addr, SocketAddrV4};
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, services::ServeDir};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Address to start server on
    #[arg(short, long)]
    address: Option<SocketAddrV4>,
}

async fn not_found_handler() -> impl axum::response::IntoResponse {
    NotFoundTemplate
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let pool =
        PgPool::connect(
                &std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL should be defined, example: postgres://postgres:shajarah-dev@localhost:5445/postgres")
            )
            .await
            .expect("Failed to connect to DB");

    if let Err(e) = sqlx::migrate!().run(&pool).await {
        log::error!("Failed to migrate DB: {e}");

        panic!("Failed to migrate DB");
    }

    let config = match Config::load_config() {
        Ok(config) => config,
        Err(err) => match &err {
            ConfigError::IoError(err) if err.kind() == std::io::ErrorKind::NotFound => {
                log::warn!("GENERATING CONFIG FILE WITH SECRET");

                let mut secret = [0u8; 64];
                rand::thread_rng().fill(&mut secret);

                let cookies_secret = String::from_utf8_lossy(&secret).to_string();

                let totp_encryption_key =
                    aes_gcm::Aes256Gcm::generate_key(rand::thread_rng()).to_vec();

                let config = Config {
                    cookie_secret: cookies_secret,
                    email_config: None,
                    totp_encryption_key,
                    base_url: "http://localhost:3030".parse().unwrap(),
                };

                let config_str =
                    toml::to_string(&config).expect("Serialize config struct to toml string");

                std::fs::write("config.toml", config_str)
                    .expect("writing config toml string to config.toml");

                config
            }
            _ => {
                panic!("{err:#?}");
            }
        },
    };

    let (email_sender, mut email_receiver) = tokio::sync::mpsc::channel::<EmailMessage>(10);

    if let Some(email_config) = config.email_config {
        tokio::spawn(async move {
            let creds = Credentials::new(
                email_config.credentials.username.clone(),
                email_config.credentials.password,
            );

            #[cfg(debug_assertions)]
            let mailer = SmtpTransport::builder_dangerous(&email_config.smtp_server)
                .port(1025)
                .credentials(creds)
                .build();

            #[cfg(not(debug_assertions))]
            let mailer = SmtpTransport::relay(&email_config.smtp_server)
                .unwrap()
                .credentials(creds)
                .build();

            while let Some(email_message) = email_receiver.recv().await {
                let Ok(m) = Message::builder()
                    .from(email_config.credentials.username.parse().unwrap())
                    .to(lettre::message::Mailbox {
                        name: None,
                        email: email_message.to.parse().unwrap(),
                    })
                    .subject("Invite")
                    .header(ContentType::TEXT_PLAIN)
                    .body(email_message.content)
                else {
                    log::error!("Failed to send email");
                    continue;
                };

                if let Err(e) = mailer.send(&m) {
                    log::error!("Could not send email: {e:?}");
                }
            }
        });
    }

    let totp_encryption_key =
        aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_exact_iter(config.totp_encryption_key)
            .expect("Slice must be the same length as the array");

    let app_state = AppState {
        inner: Arc::new(InnerAppState {
            db_pool: pool,
            cookies_secret: tower_cookies::Key::from(config.cookie_secret.as_bytes()),
            email_sender,
            base_url: config.base_url,
            totp_encryption_key,
        }),
    };

    let mut app = Router::new()
        .route("/admin", get(admin_page))
        .route("/admin/login", get(admin_login_page))
        .route("/admin/register", get(admin_register_page))
        .route("/login", get(members_login_page))
        .route("/user", get(user_page))
        .route("/add", get(add_request_page))
        .route("/invite/:id", get(invite_reply_page))
        .route("/api/members", get(get_members).post(add_member))
        .route("/api/members/:id", put(edit_member).delete(delete_member))
        .route("/api/members/flat", get(get_members_flat))
        .route("/api/members/export", get(export_members))
        .route("/api/members/import", post(upload_members_csv))
        .route("/api/members/add-request", post(request_add_member))
        .route("/api/members/approve/:id", put(approve_member_request))
        .route(
            "/api/members/disapprove/:id",
            put(disapprove_member_request),
        )
        .route("/api/members/invite/", get(get_member_invites))
        .route("/api/members/invite/:id", post(invite_member))
        .route("/api/users/invite/accept/:id", put(accept_member_invite))
        .route("/api/users/invite/decline/:id", put(decline_member_invite))
        .route("/api/users/invite/verify/:id", put(verify_totp))
        .route("/api/users/logout", get(logout))
        .route("/api/users/admin/login", post(admin_login))
        .route("/api/users/login", post(member_login))
        .route("/api/users/me", get(me))
        .route("/api/users", post(create_user));

    if let Ok(dist) = std::env::var("SHAJARAH_DIST") {
        app = app.nest_service(
            "/",
            ServeDir::new(dist).not_found_service(not_found_handler.into_service()),
        );
    } else if let Some(dist) = option_env!("SHAJARAH_DIST") {
        app = app.nest_service(
            "/",
            ServeDir::new(dist).not_found_service(not_found_handler.into_service()),
        );
    }

    let app = app
        .layer(
            CorsLayer::new()
                .allow_origin(["https://shajarah.bksalman.com"
                    .parse::<HeaderValue>()
                    .unwrap()])
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]),
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            refresh_session,
        ))
        .layer(CookieManagerLayer::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024 /* 25mb */))
        .with_state(app_state);

    let address = cli
        .address
        .unwrap_or(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3030));

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    log::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
