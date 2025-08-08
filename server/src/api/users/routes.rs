use aes_gcm::aead::Aead;
use aes_gcm::AeadCore;
use aes_gcm::KeyInit;
use argon2::{password_hash::SaltString, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::Path;
use chrono::Duration;
use chrono::Utc;
use garde::Validate;
use rand::Rng;
use sqlx::prelude::FromRow;
use std::sync::Arc;
use tower_cookies::cookie::time::OffsetDateTime;
use tower_cookies::Cookie;
use uuid::Uuid;

use argon2::Argon2;
use axum::{extract::State, Json};
use tower_cookies::Cookies;

use crate::api::members::models::{InviteStatus, MemberInvite, MemberRow};
use crate::Gender;
use crate::{
    api::sessions::{models::CreateSession, SESSION_COOKIE_NAME},
    api::users::{
        models::{AdminLogin, CreateUser, UserResponse, UserResponseBrief, UserRole},
        UsersError,
    },
    auth::AuthExtractor,
    InnerAppState,
};

use super::models::MemberLogin;
use super::models::VerifyTOTP;

#[axum::debug_handler]
pub async fn admin_login(
    State(state): State<Arc<InnerAppState>>,
    cookies: Cookies,
    Json(payload): Json<AdminLogin>,
) -> Result<(), UsersError> {
    payload.validate()?;

    let mut tx = state.db_pool.begin().await?;

    if let Some(session_id) = cookies
        .private(&state.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        if let Some(session) = sqlx::query!(
            r#"
SELECT id from sessions
WHERE sessions.id = $1
            "#,
            Uuid::parse_str(session_id.value()).map_err(|e| {
                log::error!("{e}");
                UsersError::InternalServerError
            })?
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            log::error!("user already logged in with session: {session:?}");
            return Err(UsersError::AlreadyLoggedIn);
        }
    }

    let argon2 = Argon2::default();

    #[derive(FromRow)]
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
        payload.email
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(UsersError::UserNotFound);
    };

    let Some(ref user_password) = user.password else {
        return Err(UsersError::InvalidCredentials);
    };

    let parsed_password = PasswordHash::new(&user_password)?;

    if argon2
        .verify_password(payload.password.as_bytes(), &parsed_password)
        .is_err()
    {
        return Err(UsersError::InvalidCredentials);
    }

    let now = Utc::now();
    let time_now = OffsetDateTime::now_utc();

    let new_session = CreateSession {
        id: Uuid::new_v4(),
        user_id: user.id,
        created_at: now,
        expires_at: now + Duration::days(2),
    };

    #[derive(FromRow)]
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

    let cookie = Cookie::build((SESSION_COOKIE_NAME, session.id.to_string()))
        .path("/")
        .expires(time_now + time::Duration::days(2))
        .http_only(true);

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true);

    let cookie = cookie.build();

    cookies.private(&state.cookies_secret).add(cookie);

    tx.commit().await?;

    Ok(())
}

#[axum::debug_handler]
pub async fn member_login(
    State(state): State<Arc<InnerAppState>>,
    cookies: Cookies,
    Json(payload): Json<MemberLogin>,
) -> Result<(), UsersError> {
    payload.validate()?;

    let mut tx = state.db_pool.begin().await?;

    if let Some(session_id) = cookies
        .private(&state.cookies_secret)
        .get(SESSION_COOKIE_NAME)
    {
        if let Some(session) = sqlx::query!(
            r#"
SELECT id from sessions
WHERE sessions.id = $1
            "#,
            Uuid::parse_str(session_id.value()).map_err(|e| {
                log::error!("{e}");
                UsersError::InternalServerError
            })?
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            log::error!("user already logged in with session: {session:?}");
            return Err(UsersError::AlreadyLoggedIn);
        }
    }

    #[derive(FromRow)]
    pub struct UserRow {
        pub id: Uuid,
        pub totp_secret: Option<Vec<u8>>,
    }

    let Some(user) = sqlx::query_as!(
        UserRow,
        r#"
SELECT users.id, users.totp_secret FROM users
WHERE users.email = $1
        "#,
        payload.email
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(UsersError::UserNotFound);
    };

    let Some(encrypted_totp_secret) = user.totp_secret else {
        return Err(UsersError::InvalidInvitation);
    };

    let cipher = aes_gcm::Aes256Gcm::new(&state.totp_encryption_key);

    let (nonce, encrypted_totp_secret) = encrypted_totp_secret.split_at(12);

    let decrypted_totp_secret = cipher.decrypt(
        nonce
            .try_into()
            .map_err(|_e| UsersError::InternalServerError)?,
        encrypted_totp_secret,
    )?;

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Raw(decrypted_totp_secret).to_bytes()?,
        None,
        format!(
            "totp@{}",
            state
                .base_url
                .host()
                .ok_or(UsersError::InternalServerError)?
        ),
    )?;
    let token = totp.generate_current()?;

    if token != payload.totp {
        return Err(UsersError::InvalidCredentials);
    }

    let now = Utc::now();
    let time_now = OffsetDateTime::now_utc();

    let new_session = CreateSession {
        id: Uuid::new_v4(),
        user_id: user.id,
        created_at: now,
        expires_at: now + Duration::days(2),
    };

    #[derive(FromRow)]
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

    let cookie = Cookie::build((SESSION_COOKIE_NAME, session.id.to_string()))
        .path("/")
        .expires(time_now + time::Duration::days(2))
        .http_only(true);

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true);

    let cookie = cookie.build();

    cookies.private(&state.cookies_secret).add(cookie);

    tx.commit().await?;

    Ok(())
}

pub async fn me(auth: AuthExtractor<{ UserRole::User as u8 }>) -> Json<UserResponseBrief> {
    Json(auth.current_user)
}

pub async fn logout(
    cookies: Cookies,
    State(state): State<Arc<InnerAppState>>,
    auth: AuthExtractor<{ UserRole::User as u8 }>,
) -> Result<(), UsersError> {
    sqlx::query!(
        r#"
DELETE FROM sessions WHERE sessions.id = $1
        "#,
        auth.session_id,
    )
    .execute(&state.db_pool)
    .await?;

    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true);

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true);

    cookies.remove(cookie.build());

    Ok(())
}

pub async fn create_user(
    State(state): State<Arc<InnerAppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>, UsersError> {
    payload.validate()?;

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
        return Err(UsersError::BadRequest);
    }

    if payload.password.is_empty() || payload.email.is_empty() {
        return Err(UsersError::BadRequest);
    }

    let salt = SaltString::generate(rand::thread_rng());

    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    let user = sqlx::query_as!(
        UserResponse,
        r#"
            INSERT INTO users (id, first_name, last_name, email, password, role, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, email, role as "role: UserRole"
        "#,
        Uuid::new_v4(),
        payload.first_name,
        payload.last_name,
        payload.email,
        hashed_password,
        UserRole::Admin as _,
        Utc::now(),
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(user))
}

#[axum::debug_handler]
pub async fn accept_member_invite(
    State(state): State<Arc<InnerAppState>>,
    Path(invitation_id): Path<Uuid>,
) -> anyhow::Result<String, UsersError> {
    let mut tx = state.db_pool.begin().await?;

    let invitation = sqlx::query_as!(
        MemberInvite,
        r#"
        SELECT id, member_id, email, created_at, expires_at, status as "status: InviteStatus", totp_secret
        FROM member_invites
        WHERE id = $1
        "#,
        invitation_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(UsersError::InvalidInvitation)?;

    if invitation.status != InviteStatus::Pending {
        return Err(UsersError::InvitationAlreadyUsed);
    }

    let cipher = aes_gcm::Aes256Gcm::new(&state.totp_encryption_key);
    let nonce = aes_gcm::Aes256Gcm::generate_nonce(&mut rand::thread_rng());

    let mut totp_secret: [u8; 20] = Default::default();

    rand::thread_rng().fill(&mut totp_secret[..]);

    let encrypted_totp_secret = cipher.encrypt(&nonce, &totp_secret[..])?;

    let mut encrypted_totp_secret_with_nonce = nonce.to_vec();
    encrypted_totp_secret_with_nonce.extend(encrypted_totp_secret);

    sqlx::query!(
        r#"
            UPDATE member_invites
            SET totp_secret = $1
            WHERE id = $2
        "#,
        encrypted_totp_secret_with_nonce,
        invitation_id,
    )
    .execute(&mut *tx)
    .await?;

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Raw(totp_secret.to_vec()).to_bytes()?,
        None,
        format!(
            "totp@{}",
            state
                .base_url
                .host()
                .ok_or(UsersError::InternalServerError)?
        ),
    )?;

    tx.commit().await?;

    Ok(totp.get_qr_base64().map_err(|e| UsersError::TOTPQr(e))?)
}

pub async fn decline_member_invite(
    State(state): State<Arc<InnerAppState>>,
    Path(invite_id): Path<Uuid>,
) -> anyhow::Result<(), UsersError> {
    let update = sqlx::query!(
        r#"
UPDATE member_invites
SET status = $1
WHERE id = $2;
"#,
        InviteStatus::Accepted as _,
        invite_id,
    )
    .execute(&state.db_pool)
    .await?;

    if update.rows_affected() < 1 {
        return Err(UsersError::BadRequest);
    }

    Ok(())
}

pub async fn verify_totp(
    State(state): State<Arc<InnerAppState>>,
    Path(invitation_id): Path<Uuid>,
    Json(payload): Json<VerifyTOTP>,
) -> anyhow::Result<(), UsersError> {
    payload.validate()?;

    let mut tx = state.db_pool.begin().await?;

    let Some(member_invite) = sqlx::query_as!(
        MemberInvite,
        r#"
            UPDATE member_invites
            SET status = 'accepted'
            WHERE status = 'pending' AND id = $1
            RETURNING
             id,
                    member_id,
                    email,
                    status as "status: InviteStatus",
                    created_at,
                    expires_at,
                    totp_secret;
        "#,
        invitation_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Err(UsersError::InvalidInvitation);
    };

    let Some(encrypted_totp_secret) = member_invite.totp_secret else {
        return Err(UsersError::InvalidInvitation);
    };

    let cipher = aes_gcm::Aes256Gcm::new(&state.totp_encryption_key);

    let (nonce, split_encrypted_totp_secret) = encrypted_totp_secret.split_at(12);

    let decrypted_totp_secret = cipher.decrypt(
        nonce
            .try_into()
            .map_err(|_e| UsersError::InternalServerError)?,
        split_encrypted_totp_secret,
    )?;

    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        totp_rs::Secret::Raw(decrypted_totp_secret).to_bytes()?,
        None,
        format!(
            "totp@{}",
            state
                .base_url
                .host()
                .ok_or(UsersError::InternalServerError)?
        ),
    )?;
    let token = totp.generate_current()?;

    if token != payload.totp_code {
        return Err(UsersError::InvalidCredentials);
    }

    let member = sqlx::query_as!(
        MemberRow,
        r#"
                SELECT id,
                name,
                last_name,
                gender as "gender: Gender",
                birthday,
                image,
                image_type,
                personal_info,
                mother_id,
                father_id FROM members
                WHERE id = $1;
            "#,
        member_invite.member_id
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
    INSERT INTO users (id, first_name, last_name, email, totp_secret, role, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        Uuid::new_v4(),
        member.name,
        member.last_name,
        member_invite.email,
        encrypted_totp_secret,
        UserRole::User as _,
        Utc::now(),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
