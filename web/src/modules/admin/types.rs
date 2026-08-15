use dioxus::prelude::*;
use indexmap::IndexMap;
use jiff::Zoned;

use crate::modules::member::types::Gender;

#[derive(Default, Clone, Store, PartialEq)]
pub struct MemberFormData {
    pub name: String,
    pub last_name: String,
    pub gender: Option<Gender>,
    pub birthday: Option<Zoned>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
}
