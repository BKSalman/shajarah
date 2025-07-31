use anyhow::anyhow;
use chrono::{DateTime, Utc};
use garde::Validate;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Gender;

#[derive(Deserialize, Serialize)]
pub struct CreateMember {
    pub name: String,
    pub last_name: String,
    pub gender: Gender,
    pub birthday: chrono::DateTime<chrono::Utc>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    /// Generic info about family member
    /// a map is used to make it dynamic and hold any kind of personal information
    pub info: Option<IndexMap<String, serde_json::Value>>,
}

#[derive(Default)]
pub struct CreateMemberBuilder {
    name: Option<String>,
    last_name: Option<String>,
    gender: Option<Gender>,
    birthday: Option<chrono::DateTime<chrono::Utc>>,
    mother_id: Option<i64>,
    father_id: Option<i64>,
    image: Option<Vec<u8>>,
    image_type: Option<String>,
    info: Option<IndexMap<String, serde_json::Value>>,
}

impl CreateMemberBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn last_name(&mut self, last_name: String) -> &mut Self {
        self.last_name = Some(last_name);
        self
    }

    pub fn gender(&mut self, gender: Gender) -> &mut Self {
        self.gender = Some(gender);
        self
    }

    pub fn birthday(&mut self, birthday: chrono::DateTime<chrono::Utc>) -> &mut Self {
        self.birthday = Some(birthday);
        self
    }

    pub fn mother_id(&mut self, mother_id: i64) -> &mut Self {
        self.mother_id = Some(mother_id);
        self
    }

    pub fn father_id(&mut self, father_id: i64) -> &mut Self {
        self.father_id = Some(father_id);
        self
    }

    pub fn image(&mut self, image: Vec<u8>) -> &mut Self {
        self.image = Some(image);
        self
    }

    pub fn image_type(&mut self, image_type: String) -> &mut Self {
        self.image_type = Some(image_type);
        self
    }

    pub fn info(&mut self, info: IndexMap<String, serde_json::Value>) -> &mut Self {
        self.info = Some(info);
        self
    }

    pub fn build(self) -> anyhow::Result<CreateMember> {
        let name = self.name.ok_or(anyhow!("name field was not provided"))?;
        let last_name = self
            .last_name
            .ok_or(anyhow!("last_name field was not provided"))?;
        let gender = self
            .gender
            .ok_or(anyhow!("gender field was not provided"))?;
        let birthday = self
            .birthday
            .ok_or(anyhow!("birthday field was not provided"))?;

        if self.image.is_some() != self.image_type.is_some() {
            return Err(anyhow!("image or image_type was not added"));
        }

        Ok(CreateMember {
            name,
            last_name,
            gender,
            birthday,
            mother_id: self.mother_id,
            father_id: self.father_id,
            image: self.image,
            info: self.info,
            image_type: self.image_type,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMember {
    pub id: i64,
    pub name: Option<String>,
    pub last_name: Option<String>,
    pub gender: Option<Gender>,
    pub birthday: Option<chrono::DateTime<chrono::Utc>>,
    pub mother_id: Option<i64>,
    pub father_id: Option<i64>,
    pub info: Option<IndexMap<String, serde_json::Value>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

#[derive(Default)]
pub struct UpdateMemberBuilder {
    name: Option<String>,
    last_name: Option<String>,
    gender: Option<Gender>,
    birthday: Option<chrono::DateTime<chrono::Utc>>,
    mother_id: Option<i64>,
    pub remove_mother_id: bool,
    father_id: Option<i64>,
    pub remove_father_id: bool,
    info: Option<IndexMap<String, serde_json::Value>>,
    pub remove_info: bool,
    image: Option<Vec<u8>>,
    image_type: Option<String>,
}

impl UpdateMemberBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn last_name(&mut self, last_name: String) -> &mut Self {
        self.last_name = Some(last_name);
        self
    }

    pub fn gender(&mut self, gender: Gender) -> &mut Self {
        self.gender = Some(gender);
        self
    }

    pub fn birthday(&mut self, birthday: chrono::DateTime<chrono::Utc>) -> &mut Self {
        self.birthday = Some(birthday);
        self
    }

    pub fn mother_id(&mut self, mother_id: i64) -> &mut Self {
        self.mother_id = Some(mother_id);
        self
    }

    pub fn remove_mother_id(&mut self, remove: bool) -> &mut Self {
        self.remove_mother_id = remove;
        self
    }

    pub fn father_id(&mut self, father_id: i64) -> &mut Self {
        self.father_id = Some(father_id);
        self
    }

    pub fn remove_father_id(&mut self, remove: bool) -> &mut Self {
        self.remove_father_id = remove;
        self
    }

    pub fn remove_info(&mut self, remove: bool) -> &mut Self {
        self.remove_info = remove;
        self
    }

    pub fn info(&mut self, info: IndexMap<String, serde_json::Value>) -> &mut Self {
        self.info = Some(info);
        self
    }

    pub fn image(&mut self, image: Vec<u8>) -> &mut Self {
        self.image = Some(image);
        self
    }

    pub fn image_type(&mut self, image_type: String) -> &mut Self {
        self.image_type = Some(image_type);
        self
    }

    pub fn build(self, id: i64) -> anyhow::Result<UpdateMember> {
        if self.image.is_some() != self.image_type.is_some() {
            return Err(anyhow!("image or image_type was not added"));
        }

        Ok(UpdateMember {
            id,
            name: self.name,
            last_name: self.last_name,
            gender: self.gender,
            birthday: self.birthday,
            mother_id: self.mother_id,
            father_id: self.father_id,
            image: self.image,
            image_type: self.image_type,
            info: self.info,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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

#[derive(Debug, sqlx::FromRow)]
pub struct MemberRowWithParents {
    pub id: i64,
    pub name: String,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemberResponse {
    pub id: i64,
    pub name: String,
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

/// non-recursive MemberResponse
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemberResponseBrief {
    pub id: i64,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<DateTime<Utc>>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RequestedMemberRow {
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
    pub status: RequestStatus,
}

#[derive(Debug, sqlx::FromRow)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestedMemberResponseBrief {
    pub id: Uuid,
    pub name: String,
    pub gender: Gender,
    pub birthday: Option<DateTime<Utc>>,
    pub last_name: String,
    pub father_id: Option<i64>,
    pub mother_id: Option<i64>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub image: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub status: RequestStatus,
}

#[derive(Default, Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "request_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    #[default]
    Pending,
    Approved,
    Disapproved,
}

#[derive(Default, Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "invite_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum InviteStatus {
    #[default]
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemberInvite {
    pub id: Uuid,
    pub member_id: i64,
    pub email: String,
    pub status: InviteStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub totp_secret: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemberInviteResponse {
    pub id: Uuid,
    pub member_id: i64,
    pub email: String,
    pub status: InviteStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateMemberInvite {
    #[garde(email)]
    pub email: String,
}
