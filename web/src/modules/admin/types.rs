use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

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
