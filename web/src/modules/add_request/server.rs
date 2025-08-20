use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use indexmap::IndexMap;

use crate::{
    modules::{
        add_request::types::{RequestedMemberResponse, RequestedMemberRowWithParents},
        member::types::Gender,
    },
    server::get_state,
};

#[server]
pub async fn add_request(
    name: String,
    last_name: String,
    gender: Gender,
    birthday: Option<DateTime<Utc>>,
    father_id: Option<i64>,
    mother_id: Option<i64>,
    info: HashMap<String, String>,
    image: Option<Vec<u8>>,
    image_type: Option<String>,
) -> ServerFnResult<()> {
    use crate::server::get_state;

    let state = get_state().await?;

    let Ok(info) = serde_json::value::to_value(info) else {
        return Err(ServerFnError::new("Something went wrong"));
    };

    sqlx::query!(
        r#"
            INSERT INTO member_add_requests (id, name, gender, birthday, last_name, father_id, mother_id, image, image_type, personal_info, submitted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        uuid::Uuid::new_v4(), name, gender as _, birthday, last_name, father_id, mother_id,
        image, image_type, info, Utc::now().naive_utc(),
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[server]
pub async fn requested_members() -> ServerFnResult<Vec<RequestedMemberResponse>> {
    let state = get_state().await?;

    let recs: Vec<RequestedMemberRowWithParents> = sqlx::query_as(
        r#"
        SELECT
            m.id,
            m.name,
            m.gender,
            m.birthday,
            m.last_name,
            m.image,
            m.image_type,
            m.personal_info,
            m.status,
            mother.id as mother_id,
            mother.name AS mother_name,
            mother.gender AS mother_gender,
            mother.birthday AS mother_birthday,
            mother.last_name AS mother_last_name,
            father.id as father_id,
            father.name AS father_name,
            father.gender AS father_gender,
            father.birthday AS father_birthday,
            father.last_name AS father_last_name
        FROM
            member_add_requests m
        LEFT JOIN
            members mother ON m.mother_id = mother.id
        LEFT JOIN
            members father ON m.father_id = father.id
        ORDER BY
            m.submitted_at DESC,
            m.name ASC
            "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    let requested_members: Vec<RequestedMemberResponse> = recs
        .into_iter()
        .map(|m| RequestedMemberResponse {
            id: m.id,
            name: m.name,
            gender: m.gender,
            birthday: m.birthday,
            last_name: m.last_name,
            father_id: m.father_id,
            father_name: m.father_name,
            mother_id: m.mother_id,
            mother_name: m.mother_name,
            personal_info: m.personal_info.as_ref().and_then(|p| {
                p.as_object().map(|o| {
                    o.into_iter()
                        .map(|(k, v)| (k.to_string(), v.as_str().unwrap_or("").to_string()))
                        .rev()
                        .collect::<IndexMap<String, String>>()
                })
            }),
            image: m.image,
            image_type: m.image_type,
            status: m.status,
        })
        .collect();

    Ok(requested_members)
}
