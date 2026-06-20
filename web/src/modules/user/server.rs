use dioxus::prelude::*;
use uuid::Uuid;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::{AuthError, AuthExtractor};
    pub use crate::middleware::sessions::SESSION_COOKIE_NAME;
    pub use crate::server::AppState;
    pub use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    pub use axum::Extension;
    pub use chrono::Utc;
    pub use garde::Validate as _;
    pub use sha2::Digest as _;
    pub use sha2::Sha256;
}

#[cfg(feature = "server")]
use server_imports::*;

use crate::modules::user::types::{LoginData, UserResponseBrief};

use super::types::{RegisterData, UserRole};

#[get("/api/v1/user", user: Result<AuthExtractor<{ UserRole::User as u8 }>, AuthError>)]
pub async fn get_user() -> Result<UserResponseBrief> {
    Ok(user.or_unauthorized("unauthorized")?.current_user)
}

#[post("/api/v1/user", Extension(state): Extension<AppState>)]
pub async fn register_user(invite_token: Uuid, register_data: RegisterData) -> anyhow::Result<()> {
    use crate::modules::invite::types::InviteRow;

    let mut hasher = Sha256::default();
    hasher.update(invite_token.to_string().as_bytes());
    let token_hash = hex::encode(hasher.finalize());

    let mut tx = state.db_pool.begin().await?;

    tracing::info!("{invite_token}");

    if sqlx::query_as!(
        InviteRow,
        r#"
            SELECT * FROM user_invites
            WHERE token_hash = $1 AND expires_at > $2 AND used_at IS NULL
        "#,
        token_hash,
        Utc::now()
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_none()
    {
        return Err(anyhow::anyhow!("Invalid invite"));
    };

    register_data.validate()?;

    let RegisterData {
        first_name: Some(first_name),
        last_name: Some(last_name),
        email: Some(email),
        password: Some(password),
        confirm_password: _,
    } = register_data
    else {
        // this shouldn't run since we validate that all fields have values
        return Err(anyhow::anyhow!("Something went wrong"));
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Something went wrong"))?
        .to_string();

    let user = sqlx::query!(
        r#"
            INSERT INTO users (id, first_name, last_name, email, password, role, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id;
        "#,
        Uuid::new_v4(),
        first_name,
        last_name,
        email,
        hashed_password,
        UserRole::User as _,
        Utc::now(),
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query_as!(
        InviteRow,
        r#"
            UPDATE user_invites
            SET used_at = $1, accepted_by = $2
            WHERE token_hash = $3 AND expires_at > $4 AND used_at = NULL
        "#,
        Utc::now(),
        user.id,
        token_hash,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[post("/api/v1/user/logout", Extension(state): Extension<AppState>, cookies: tower_cookies::Cookies)]
pub async fn logout_user() -> anyhow::Result<()> {
    if let Some(session_id) = cookies
        .private(&state.config.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        sqlx::query!(
            r#"
                DELETE from sessions
                WHERE sessions.id = $1
            "#,
            Uuid::parse_str(session_id.value()).map_err(|e| { anyhow::anyhow!("Bad Request") })?
        )
        .execute(&state.db_pool)
        .await?;

        let cookie = tower_cookies::Cookie::build(SESSION_COOKIE_NAME)
            .path("/")
            .http_only(true)
            .build();

        cookies.private(&state.config.cookies_secret).remove(cookie);
    }

    Ok(())
}

#[post("/api/v1/user/login", Extension(state): Extension<AppState>, cookies: tower_cookies::Cookies)]
pub async fn login_user(login_data: LoginData) -> anyhow::Result<()> {
    use crate::middleware::sessions::{SESSION_COOKIE_NAME, types::CreateSession};
    use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
    use chrono::Utc;
    use uuid::Uuid;

    login_data.validate()?;

    let LoginData {
        email: Some(email),
        password: Some(password),
    } = login_data
    else {
        return Err(anyhow::anyhow!("Something went wrong"));
    };

    let mut tx = state.db_pool.begin().await?;

    if let Some(session_id) = cookies
        .private(&state.config.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        if sqlx::query!(
            r#"
                SELECT id from sessions
                WHERE sessions.id = $1
            "#,
            Uuid::parse_str(session_id.value()).map_err(|e| { anyhow::anyhow!("Bad Request") })?
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some()
        {
            return Err(anyhow::anyhow!("Bad Request"));
        }
    }

    let argon2 = Argon2::default();

    #[derive(sqlx::FromRow)]
    pub struct UserRow {
        pub id: Uuid,
        pub password: Option<String>,
    }

    let Some(user) = sqlx::query_as!(
        UserRow,
        r#"
            SELECT users.id, users.password FROM users
            WHERE users.email = $1
        "#,
        email
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(anyhow::anyhow!("Bad Request"));
    };

    let Some(ref user_password) = user.password else {
        return Err(anyhow::anyhow!("Invalid credentials"));
    };

    let parsed_password =
        PasswordHash::new(&user_password).map_err(|e| anyhow::anyhow!("Something went wrong"))?;

    if argon2
        .verify_password(password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(anyhow::anyhow!("Inavlid credentials"));
    }

    let now = Utc::now();
    let time_now = tower_cookies::cookie::time::OffsetDateTime::now_utc();

    let new_session = CreateSession {
        id: Uuid::new_v4(),
        user_id: user.id,
        created_at: now,
        expires_at: now + chrono::Duration::days(2),
    };

    #[derive(sqlx::FromRow)]
    struct SessionRow {
        id: Uuid,
    }

    let session = sqlx::query_as!(
        SessionRow,
        r#"
            INSERT INTO sessions (id, user_id, created_at, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING sessions.id
        "#,
        new_session.id,
        new_session.user_id,
        new_session.created_at,
        new_session.expires_at,
    )
    .fetch_one(&mut *tx)
    .await?;

    let cookie = tower_cookies::Cookie::build((SESSION_COOKIE_NAME, session.id.to_string()))
        .path("/")
        .expires(time_now + tower_cookies::cookie::time::Duration::days(2))
        .http_only(true);

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true);

    let cookie = cookie.build();

    cookies.private(&state.config.cookies_secret).add(cookie);

    tx.commit().await?;

    Ok(())
}
