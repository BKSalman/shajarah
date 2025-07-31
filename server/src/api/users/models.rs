use chrono::DateTime;
use garde::Validate;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "UserRole", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin = 0,
    User = 1,
}

#[derive(Debug)]
pub struct ProfileImage {
    pub id: Uuid,
    pub path: String,
    pub content_type: String,
    pub user_id: Uuid,
    pub updated_at: Option<DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[garde(skip)]
    pub first_name: String,
    #[garde(skip)]
    pub last_name: String,
    #[garde(email)]
    pub email: String,
    // TODO: add password rules
    #[garde(skip)]
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateUserResponse {
    pub user_id: Uuid,
}

#[derive(Deserialize, Validate)]
pub struct AdminLogin {
    #[garde(email)]
    pub email: String,
    // TODO: add password rules
    #[garde(skip)]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct MemberLogin {
    #[garde(email)]
    pub email: String,
    #[garde(skip)]
    pub totp: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponseBrief {
    pub id: Uuid,
    pub first_name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserClaims {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserToken {
    pub access_token: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct VerifyTOTP {
    #[garde(length(min = 6, max = 6))]
    pub totp_code: String,
}
