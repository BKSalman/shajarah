use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::modules::member::types::Gender;

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
