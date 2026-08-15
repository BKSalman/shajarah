use dioxus::prelude::*;
use garde::Validate;
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "UserRole", rename_all = "snake_case")
)]
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
    pub updated_at: Option<Zoned>,
}

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[garde(skip)]
    pub first_name: String,
    #[garde(skip)]
    pub last_name: String,
    #[garde(email)]
    pub email: String,
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

#[cfg_attr(feature = "server", derive(sqlx::prelude::FromRow))]
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Store, garde::Validate, Serialize, Deserialize, Clone)]
pub struct RegisterData {
    #[garde(required, length(min = 1))]
    pub first_name: Option<String>,
    #[garde(required, length(min = 1))]
    pub last_name: Option<String>,
    #[garde(required, email)]
    pub email: Option<String>,
    #[garde(required, length(min = 8))]
    pub password: Option<String>,
    #[garde(required, length(min = 8), matches(password))]
    pub confirm_password: Option<String>,
}

#[derive(Debug, Store, garde::Validate, Serialize, Deserialize, Clone)]
pub struct LoginData {
    #[garde(required, email)]
    pub email: Option<String>,
    #[garde(required, length(min = 8))]
    pub password: Option<String>,
}
