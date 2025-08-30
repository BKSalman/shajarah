use super::types::RegisterData;
use crate::modules::{
    admin::types::LoginData,
    user::types::{UserResponseBrief, UserRole},
};
use dioxus::prelude::*;
use garde::Validate;

#[server]
pub async fn get_admin() -> ServerFnResult<UserResponseBrief> {
    use crate::middleware::auth::AuthExtractor;
    let auth = extract::<AuthExtractor<{ UserRole::Admin as u8 }>, _>().await?;
    Ok(auth.current_user)
}

#[server]
pub async fn register_admin(register_data: RegisterData) -> ServerFnResult<()> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use chrono::Utc;
    use uuid::Uuid;

    let state = crate::server::get_state().await?;

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
        return Err(ServerFnError::new("Bad Request"));
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
        return Err(ServerFnError::new("Something went wrong"));
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ServerFnError::new("Something went wrong"))?
        .to_string();

    sqlx::query_as!(
        UserResponse,
        r#"
                    INSERT INTO users (id, first_name, last_name, email, password, role, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7);
                "#,
        Uuid::new_v4(),
        first_name,
        last_name,
        email,
        hashed_password,
        UserRole::Admin as _,
        Utc::now(),
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[server]
pub async fn login_admin(login_data: LoginData) -> ServerFnResult<()> {
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
        return Err(ServerFnError::new("Something went wrong"));
    };

    let state = crate::server::get_state().await?;
    let cookies = crate::server::get_cookies().await?;

    let mut tx = state.db_pool.begin().await?;

    if let Some(session_id) = cookies
        .private(&state.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        if sqlx::query!(
            r#"
                SELECT id from sessions
                WHERE sessions.id = $1
            "#,
            Uuid::parse_str(session_id.value())
                .map_err(|e| { ServerFnError::new("Bad Request") })?
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some()
        {
            return Err(ServerFnError::new("Bad Request"));
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
        return Err(ServerFnError::new("Bad Request"));
    };

    let Some(ref user_password) = user.password else {
        return Err(ServerFnError::new("Invalid credentials"));
    };

    let parsed_password = PasswordHash::new(&user_password)
        .map_err(|e| ServerFnError::new("Something went wrong"))?;

    if argon2
        .verify_password(password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(ServerFnError::new("Inavlid credentials"));
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

    cookies.private(&state.cookies_secret).add(cookie);

    tx.commit().await?;

    Ok(())
}

#[server]
pub async fn logout_admin() -> ServerFnResult<()> {
    use crate::middleware::sessions::SESSION_COOKIE_NAME;
    use uuid::Uuid;

    let state = crate::server::get_state().await?;
    let cookies = crate::server::get_cookies().await?;

    let mut tx = state.db_pool.begin().await?;

    if let Some(session_id) = cookies
        .private(&state.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        sqlx::query!(
            r#"
                DELETE from sessions
                WHERE sessions.id = $1
            "#,
            Uuid::parse_str(session_id.value())
                .map_err(|e| { ServerFnError::new("Bad Request") })?
        )
        .execute(&mut *tx)
        .await?;

        let cookie = tower_cookies::Cookie::build(SESSION_COOKIE_NAME)
            .path("/")
            .http_only(true)
            .build();

        cookies.private(&state.cookies_secret).remove(cookie);
    }

    Ok(())
}
