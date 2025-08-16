use chrono::{DateTime, Utc};
use dioxus::prelude::*;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::modules::member::types::MemberRowWithParents;
    pub use crate::server::get_state;
    pub use indexmap::IndexMap;
}

#[cfg(feature = "server")]
use server_imports::*;

use super::types::{ChildMember, Gender, MemberResponse, MemberResponseFlat};

#[server]
pub async fn members_flat() -> ServerFnResult<Vec<MemberResponseFlat>> {
    let state = get_state().await?;

    let recs: Vec<MemberRowWithParents> = sqlx::query_as(
        r#"
        SELECT
            m.id,
            m.name,
            m.gender,
            m.birthday,
            m.email,
            m.last_name,
            m.image,
            m.image_type,
            m.personal_info,
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
            members m
        LEFT JOIN
            members mother ON m.mother_id = mother.id
        LEFT JOIN
            members father ON m.father_id = father.id
        ORDER BY
            m.id, m.name ASC
            "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    let all_children: Vec<ChildMember> = sqlx::query_as(
        r#"
        SELECT 
            id,
            name,
            gender,
            birthday,
            last_name,
            email,
            mother_id,
            father_id
        FROM members 
        WHERE mother_id IS NOT NULL OR father_id IS NOT NULL
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    // Convert to response format
    let members: Vec<MemberResponseFlat> = recs
        .into_iter()
        .map(|m| {
            let children: Vec<ChildMember> = all_children
                .iter()
                .filter(|child| child.mother_id == Some(m.id) || child.father_id == Some(m.id))
                .cloned()
                .collect();

            MemberResponseFlat {
                id: m.id,
                name: m.name,
                gender: m.gender,
                birthday: m.birthday,
                last_name: m.last_name,
                father_id: m.father_id,
                mother_id: m.mother_id,
                father_name: m.father_name,
                mother_name: m.mother_name,
                personal_info: m.personal_info.as_ref().and_then(|p| {
                    p.as_object().map(|o| {
                        o.into_iter()
                            .map(|(k, v)| (k.to_string(), v.as_str().unwrap_or("").to_string()))
                            .collect::<IndexMap<String, String>>()
                    })
                }),
                image: m.image,
                image_type: m.image_type,
                children,
            }
        })
        .collect();

    Ok(members)
}

#[server]
pub async fn add_member(
    first_name: String,
    last_name: String,
    gender: Gender,
    birthday: DateTime<Utc>,
) -> ServerFnResult<()> {
    if first_name.is_empty() || last_name.is_empty() {
        return Err(ServerFnError::new(""));
    }

    let state = get_state().await?;
    sqlx::query!(
        r#"
            INSERT INTO members (name, last_name, gender, birthday)
            VALUES ($1, $2, $3, $4);
        "#,
        first_name,
        last_name,
        gender as _,
        birthday,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[server]
pub async fn delete_member(id: i64) -> ServerFnResult<()> {
    let state = get_state().await?;
    sqlx::query!(
        r#"
            DELETE FROM members
            WHERE id = $1
        "#,
        id,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[server(endpoint = "members")]
async fn members() -> ServerFnResult<Option<MemberResponse>> {
    Ok(Some(MemberResponse {
        id: 1,
        name: String::from("سلمان"),
        gender: Gender::Male,
        birthday: Some(Utc::now()),
        last_name: String::from("السلماني"),
        father_id: None,
        mother_id: None,
        personal_info: None,
        children: Vec::new(),
        image: None,
        image_type: None,
    }))
}
