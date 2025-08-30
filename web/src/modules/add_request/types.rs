use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::member::types::Gender;

#[derive(Debug, Clone, Store, garde::Validate, Serialize, Deserialize)]
pub struct RequestData {
    #[garde(required, length(min = 1))]
    pub name: Option<String>,
    #[garde(required, length(min = 1))]
    pub last_name: Option<String>,
    #[garde(required)]
    pub gender: Option<Gender>,
    #[garde(skip)]
    pub birthday: Option<DateTime<Utc>>,
    #[garde(required)]
    pub father_id: Option<i64>,
    #[garde(skip)]
    pub mother_id: Option<i64>,
    #[garde(skip)]
    pub info: IndexMap<String, String>,
    #[garde(skip)]
    pub image: Option<Vec<u8>>,
    #[garde(skip)]
    pub image_type: Option<String>,
}

#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "request_status", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    #[default]
    Pending,
    Approved,
    Disapproved,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RequestedMemberResponse {
    pub id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<DateTime<Utc>>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub father_name: Option<String>,
    pub mother_id: Option<i64>,
    pub mother_name: Option<String>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub status: RequestStatus,
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug, Clone, PartialEq)]
pub struct RequestedMemberRowWithParents {
    pub id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub last_name: String,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub personal_info: Option<serde_json::Value>,
    pub mother_name: Option<String>,
    pub mother_gender: Option<Gender>,
    pub mother_birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub mother_last_name: Option<String>,
    pub father_name: Option<String>,
    pub father_gender: Option<Gender>,
    pub father_birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub father_last_name: Option<String>,
    pub status: RequestStatus,
}
