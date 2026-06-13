use std::str::FromStr;

use chrono::{DateTime, Utc};
use dioxus::{
    core::{AttributeValue, IntoAttributeValue},
    prelude::*,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "gender", rename_all = "snake_case")
)]
pub enum Gender {
    Male,
    Female,
}

impl core::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Male => write!(f, "male"),
            Gender::Female => write!(f, "female"),
        }
    }
}

impl IntoAttributeValue for Gender {
    fn into_value(self) -> dioxus_core::AttributeValue {
        match self {
            Gender::Male => AttributeValue::Text(String::from("male")),
            Gender::Female => AttributeValue::Text(String::from("female")),
        }
    }
}

impl FromStr for Gender {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            _ => Err(String::from("Invalid gender")),
        }
    }
}

#[derive(Store, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MemberResponse {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub gender: Gender,
    pub birthday: Option<DateTime<Utc>>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub children: Vec<MemberResponse>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

impl MemberResponse {
    pub fn add_all_children(&mut self, all_members: &[MemberRowWithParents]) {
        self.children = all_members
            .iter()
            .filter(|m| {
                m.father_id.is_some_and(|fid| fid == self.id)
                    || m.mother_id.is_some_and(|mid| mid == self.id)
            })
            .map(|m| MemberResponse {
                id: m.id,
                name: m.name.clone(),
                full_name: m.full_name.clone().unwrap_or_else(|| m.name.clone()),
                gender: m.gender,
                birthday: m.birthday,
                last_name: m.last_name.clone(),
                father_id: m.father_id,
                mother_id: m.mother_id,
                personal_info: m.personal_info.as_ref().and_then(|p| {
                    p.as_object().map(|o| {
                        o.into_iter()
                            .map(|(k, v)| (k.to_string(), v.as_str().unwrap_or("").to_string()))
                            .rev()
                            .collect::<IndexMap<String, String>>()
                    })
                }),
                children: vec![],
                image: m.image.clone(),
                image_type: m.image_type.clone(),
            })
            .collect();
        for child in &mut self.children {
            child.add_all_children(all_members);
        }
    }
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChildMember {
    pub id: i64,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub name: String,
    pub last_name: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MemberResponseFlat {
    pub id: i64,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<DateTime<Utc>>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub father_name: Option<String>,
    pub mother_name: Option<String>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub children: Vec<ChildMember>,
}
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug)]
pub struct MemberRowWithParents {
    pub id: i64,
    pub name: String,
    pub full_name: Option<String>,
    pub gender: Gender,
    pub birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub email: Option<String>,
    pub last_name: String,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub personal_info: Option<serde_json::Value>,
    pub mother_id: Option<i64>,
    pub mother_name: Option<String>,
    pub mother_gender: Option<Gender>,
    pub mother_birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub mother_last_name: Option<String>,
    pub father_id: Option<i64>,
    pub father_name: Option<String>,
    pub father_gender: Option<Gender>,
    pub father_birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub father_last_name: Option<String>,
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberRow {
    pub id: i64,
    pub name: String,
    pub last_name: String,
    pub gender: Gender,
    pub birthday: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip)]
    pub image: Option<Vec<u8>>,
    #[serde(skip)]
    pub image_type: Option<String>,
    #[serde(skip)]
    pub personal_info: Option<serde_json::Value>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
}

#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize)]
pub struct MemberSearch {
    pub id: i64,
    pub name: String,
    pub last_name: String,
    pub full_name: Option<String>,
    pub gender: Gender,
    pub birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub father: Option<String>,
    pub grandfather: Option<String>,
    pub great_grandfather: Option<String>,
}
