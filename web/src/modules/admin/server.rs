use dioxus::prelude::*;
use garde::Validate;

use crate::modules::user::types::{LoginData, RegisterData, UserResponseBrief, UserRole};

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::{AuthError, AuthExtractor};
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
}

#[cfg(feature = "server")]
use server_imports::*;

#[get("/api/v1/admin", admin: Result<AuthExtractor<{ UserRole::Admin as u8 }>, AuthError>)]
pub async fn get_admin() -> Result<UserResponseBrief> {
    Ok(admin.or_unauthorized("Unauthorized")?.current_user)
}

#[post("/api/v1/admin", Extension(state): Extension<AppState>)]
pub async fn register_admin(register_data: RegisterData) -> anyhow::Result<()> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use uuid::Uuid;

    if sqlx::query!(
        r#"
        SELECT id, role as "role: UserRole" FROM users
        WHERE role = $1
                "#,
        UserRole::Admin as _,
    )
    .fetch_optional(&state.db_pool)
    .await?
    .is_some()
    {
        return Err(anyhow::anyhow!("Bad Request"));
    }

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

    sqlx::query_as!(
        UserResponse,
        r#"
            INSERT INTO users (id, first_name, last_name, email, password, role, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now());
        "#,
        Uuid::new_v4(),
        first_name,
        last_name,
        email,
        hashed_password,
        UserRole::Admin as _,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[post("/api/v1/admin/login", Extension(state): Extension<AppState>, cookies: tower_cookies::Cookies)]
pub async fn login_admin(login_data: LoginData) -> anyhow::Result<()> {
    use crate::middleware::sessions::{SESSION_COOKIE_NAME, types::CreateSession};
    use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
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
        && sqlx::query!(
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
        tracing::error!("existing session");
        return Err(anyhow::anyhow!("Bad Request"));
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
            WHERE users.email = $1 AND users.role = 'admin'
        "#,
        email
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        tracing::error!("admin user not found");
        return Err(anyhow::anyhow!("Bad Request"));
    };

    let Some(ref user_password) = user.password else {
        return Err(anyhow::anyhow!("Invalid credentials"));
    };

    let parsed_password =
        PasswordHash::new(user_password).map_err(|e| anyhow::anyhow!("Something went wrong"))?;

    if argon2
        .verify_password(password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(anyhow::anyhow!("Inavlid credentials"));
    }

    let time_now = tower_cookies::cookie::time::OffsetDateTime::now_utc();

    let new_session = CreateSession {
        id: Uuid::new_v4(),
        user_id: user.id,
    };

    #[derive(sqlx::FromRow)]
    struct SessionRow {
        id: Uuid,
    }

    let session = sqlx::query_as!(
        SessionRow,
        r#"
            INSERT INTO sessions (id, user_id, created_at, expires_at)
            VALUES ($1, $2, now(), now() + interval '2 days')
            RETURNING sessions.id
        "#,
        new_session.id,
        new_session.user_id,
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
