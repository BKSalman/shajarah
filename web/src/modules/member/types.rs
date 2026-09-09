use std::str::FromStr;

use dioxus::{
    core::{AttributeValue, IntoAttributeValue},
    prelude::*,
};
use indexmap::IndexMap;
use jiff::Zoned;
use serde::{Deserialize, Serialize};

use crate::modules::types::EditField;

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

impl Gender {
    /// The gender a spouse must have: this app models marriage across genders.
    pub fn opposite(self) -> Gender {
        match self {
            Gender::Male => Gender::Female,
            Gender::Female => Gender::Male,
        }
    }
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

#[cfg_attr(feature = "server", derive(sqlx::Type))]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "server",
    sqlx(type_name = "marriage_status", rename_all = "snake_case")
)]
pub enum MarriageStatus {
    #[default]
    Married,
    Separated,
}

impl core::fmt::Display for MarriageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarriageStatus::Married => write!(f, "married"),
            MarriageStatus::Separated => write!(f, "separated"),
        }
    }
}

impl IntoAttributeValue for MarriageStatus {
    fn into_value(self) -> dioxus_core::AttributeValue {
        AttributeValue::Text(self.to_string())
    }
}

impl FromStr for MarriageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "married" => Ok(MarriageStatus::Married),
            "separated" => Ok(MarriageStatus::Separated),
            _ => Err(String::from("Invalid marriage status")),
        }
    }
}

/// The Arabic label shown for a marriage state.
impl MarriageStatus {
    pub fn label(&self) -> &'static str {
        match self {
            MarriageStatus::Married => "متزوج",
            MarriageStatus::Separated => "منفصل",
        }
    }
}

/// One spouse as seen from a member. `member_id` is the member this row is a
/// spouse *of*, used to group the flat query in Rust the way [`ChildMember`] is.
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SpouseLink {
    pub member_id: i64,
    pub marriage_id: i64,
    pub id: i64,
    pub name: String,
    pub last_name: String,
    pub full_name: String,
    pub gender: Gender,
    pub status: MarriageStatus,
}

#[derive(Store, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MemberResponse {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub gender: Gender,
    pub birthday: Option<Zoned>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub children: Vec<MemberResponse>,
    pub spouses: Vec<SpouseLink>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

impl MemberResponse {
    #[cfg(feature = "server")]
    pub fn add_all_children(
        &mut self,
        all_members: &[MemberRowWithParents],
        all_spouses: &[SpouseLink],
    ) {
        use jiff::tz::TimeZone;

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
                birthday: m.birthday.map(|t| t.to_jiff().to_zoned(TimeZone::UTC)),
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
                spouses: all_spouses
                    .iter()
                    .filter(|spouse| spouse.member_id == m.id)
                    .cloned()
                    .collect(),
                image: m.image.clone(),
                image_type: m.image_type.clone(),
            })
            .collect();
        for child in &mut self.children {
            child.add_all_children(all_members, all_spouses);
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
    pub full_name: String,
    pub gender: Gender,
    pub birthday: Option<Zoned>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub father_name: Option<String>,
    pub mother_name: Option<String>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub children: Vec<ChildMember>,
    pub spouses: Vec<SpouseLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct MemberUnauthorizedResponseFlat {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub gender: Gender,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub father_name: Option<String>,
    pub mother_name: Option<String>,
    pub children: Vec<ChildMember>,
}

impl From<&MemberResponseFlat> for MemberUnauthorizedResponseFlat {
    fn from(value: &MemberResponseFlat) -> Self {
        MemberUnauthorizedResponseFlat {
            id: value.id,
            name: value.name.clone(),
            full_name: value.full_name.clone(),
            gender: value.gender,
            last_name: value.last_name.clone(),
            father_id: value.father_id,
            mother_id: value.mother_id,
            father_name: value.father_name.clone(),
            mother_name: value.mother_name.clone(),
            children: value.children.clone(),
        }
    }
}

impl From<MemberResponseFlat> for MemberUnauthorizedResponseFlat {
    fn from(value: MemberResponseFlat) -> Self {
        MemberUnauthorizedResponseFlat {
            id: value.id,
            name: value.name,
            full_name: value.full_name,
            gender: value.gender,
            last_name: value.last_name,
            father_id: value.father_id,
            mother_id: value.mother_id,
            father_name: value.father_name,
            mother_name: value.mother_name,
            children: value.children,
        }
    }
}

#[cfg(feature = "server")]
#[derive(Debug, sqlx::FromRow)]
pub struct MemberRowWithParents {
    pub id: i64,
    pub name: String,
    pub full_name: Option<String>,
    pub gender: Gender,
    pub birthday: Option<jiff_sqlx::Timestamp>,
    pub email: Option<String>,
    pub last_name: String,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub personal_info: Option<serde_json::Value>,
    pub mother_id: Option<i64>,
    pub mother_name: Option<String>,
    pub mother_gender: Option<Gender>,
    pub mother_birthday: Option<jiff_sqlx::Timestamp>,
    pub mother_last_name: Option<String>,
    pub father_id: Option<i64>,
    pub father_name: Option<String>,
    pub father_gender: Option<Gender>,
    pub father_birthday: Option<jiff_sqlx::Timestamp>,
    pub father_last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberRow {
    pub id: i64,
    pub name: String,
    pub last_name: String,
    pub gender: Gender,
    pub birthday: Option<Zoned>,
    #[serde(skip)]
    pub image: Option<Vec<u8>>,
    #[serde(skip)]
    pub image_type: Option<String>,
    #[serde(skip)]
    pub personal_info: Option<serde_json::Value>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberSearch {
    pub id: i64,
    pub name: String,
    pub last_name: String,
    pub full_name: Option<String>,
    pub gender: Gender,
    pub birthday: Option<Zoned>,
    pub father: Option<String>,
    pub grandfather: Option<String>,
    pub great_grandfather: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Store)]
#[serde(default)]
pub struct EditMember {
    pub name: Option<String>,
    pub last_name: Option<String>,
    pub father_id: EditField<i64>,
    pub mother_id: EditField<i64>,
    pub gender: Option<Gender>,
    pub birthday: EditField<Zoned>,
    pub personal_info: EditField<IndexMap<String, String>>,
}

/// Orders a couple into the `(husband_id, wife_id)` column pair. `None` when the
/// two are the same person or share a gender — a marriage the schema rejects.
pub fn spouse_columns(a: (i64, Gender), b: (i64, Gender)) -> Option<(i64, i64)> {
    if a.0 == b.0 {
        return None;
    }

    match (a.1, b.1) {
        (Gender::Male, Gender::Female) => Some((a.0, b.0)),
        (Gender::Female, Gender::Male) => Some((b.0, a.0)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spouse_columns_orders_by_gender() {
        assert_eq!(
            spouse_columns((1, Gender::Male), (2, Gender::Female)),
            Some((1, 2))
        );
        assert_eq!(
            spouse_columns((2, Gender::Female), (1, Gender::Male)),
            Some((1, 2))
        );
    }

    #[test]
    fn spouse_columns_rejects_impossible_pairs() {
        assert_eq!(spouse_columns((1, Gender::Male), (2, Gender::Male)), None);
        assert_eq!(
            spouse_columns((1, Gender::Female), (2, Gender::Female)),
            None
        );
        assert_eq!(spouse_columns((1, Gender::Male), (1, Gender::Male)), None);
    }
}
