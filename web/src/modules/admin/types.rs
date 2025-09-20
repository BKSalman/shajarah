use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::modules::member::types::Gender;

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

#[derive(Default, Clone, Store, PartialEq)]
pub struct MemberFormData {
    pub name: String,
    pub last_name: String,
    pub gender: Option<Gender>,
    pub birthday: Option<DateTime<Utc>>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
}

#[derive(Debug, Default, Clone, Store, PartialEq)]
pub struct EditMemberFormData {
    pub id: i64,
    pub name: Option<String>,
    pub last_name: Option<String>,
    pub gender: Option<Gender>,
    pub birthday: Option<DateTime<Utc>>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
}
