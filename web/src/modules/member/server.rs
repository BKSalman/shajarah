use chrono::{DateTime, Utc};
use dioxus::prelude::*;
#[cfg(feature = "server")]
mod server_imports {
    pub use crate::modules::member::types::MemberRowWithParents;
    pub use crate::server::get_state;
    pub use indexmap::IndexMap;
}
use crate::modules::member::types::MemberRow;

use super::types::{ChildMember, Gender, MemberResponse, MemberResponseFlat};
#[cfg(feature = "server")]
use server_imports::*;

#[server(endpoint = "members")]
pub async fn members() -> ServerFnResult<Option<MemberResponse>> {
    let state = get_state().await?;

    let recs = sqlx::query_as!(
        MemberRowWithParents,
        r#"
            SELECT
                m.id,
                m.name,
                m.gender as "gender: Gender",
                m.birthday,
                m.last_name,
                m.image,
                m.image_type,
                m.personal_info,
                m.email,
                mother.id AS mother_id,
                mother.name AS mother_name,
                mother.gender AS "mother_gender: Gender",
                mother.birthday AS mother_birthday,
                mother.last_name AS mother_last_name,
                father.id AS father_id,
                father.name AS father_name,
                father.gender AS "father_gender: Gender",
                father.birthday AS father_birthday,
                father.last_name AS father_last_name
            FROM
                members m
            LEFT JOIN
                members mother ON m.mother_id = mother.id
            LEFT JOIN
                members father ON m.father_id = father.id;
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    if recs.is_empty() {
        return Ok(None);
    }

    let Some(root) = recs
        .iter()
        .find(|rec| rec.father_id.is_none() && rec.mother_id.is_none())
    else {
        return Err(ServerFnError::new("No root member"));
    };

    let mut root = MemberResponse {
        id: root.id,
        name: root.name.clone(),
        gender: root.gender,
        birthday: root.birthday,
        last_name: root.last_name.clone(),
        father_id: None,
        mother_id: None,
        personal_info: root.personal_info.as_ref().and_then(|p| {
            p.as_object().map(|o| {
                o.into_iter()
                    .map(|(k, v)| (k.to_string(), v.as_str().unwrap_or("").to_string()))
                    .rev()
                    .collect::<IndexMap<String, String>>()
            })
        }),
        children: Vec::new(),
        image: root.image.clone(),
        image_type: root.image_type.clone(),
    };

    root.add_all_children(&recs);

    Ok(Some(root))
}

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
    father_id: Option<i64>,
    mother_id: Option<i64>,
    gender: Gender,
    birthday: Option<DateTime<Utc>>,
) -> ServerFnResult<()> {
    if first_name.is_empty() || last_name.is_empty() {
        return Err(ServerFnError::new(""));
    }

    let state = get_state().await?;

    sqlx::query!(
        r#"
            INSERT INTO members (name, last_name, father_id, mother_id, gender, birthday)
            VALUES ($1, $2, $3, $4, $5, $6);
        "#,
        first_name,
        last_name,
        father_id,
        mother_id,
        gender as _,
        birthday,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[server]
pub async fn edit_member(
    member_id: i64,
    first_name: Option<String>,
    last_name: Option<String>,
    father_id: Option<i64>,
    mother_id: Option<i64>,
    gender: Option<Gender>,
    birthday: Option<DateTime<Utc>>,
) -> ServerFnResult<()> {
    if first_name.is_some()
        || last_name.is_some()
        || father_id.is_some()
        || mother_id.is_some()
        || gender.is_some()
        || birthday.is_some()
    {
        let state = get_state().await?;

        let mut query = sqlx::QueryBuilder::new("UPDATE members SET ");
        let mut sep = query.separated(", ");

        if let Some(first_name) = first_name {
            sep.push_unseparated("name = ").push_bind(first_name);
        }

        if let Some(last_name) = last_name {
            sep.push_unseparated("last_name = ").push_bind(last_name);
        }

        if let Some(gender) = gender {
            sep.push_unseparated("gender = ").push_bind(gender);
        }

        if let Some(birthday) = birthday {
            sep.push_unseparated("birthday = ").push_bind(birthday);
        }

        if let Some(father_id) = father_id {
            sep.push_unseparated("father_id = ").push_bind(father_id);
        }

        if let Some(mother_id) = mother_id {
            sep.push_unseparated("mother_id = ").push_bind(mother_id);
        }

        query.push(" WHERE id = ").push_bind(member_id);

        query.build().execute(&state.db_pool).await?;
    }

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

#[server]
pub async fn upload_members_csv(csv_str: String) -> ServerFnResult<()> {
    let _ = crate::modules::admin::server::get_admin().await?;
    let state = crate::server::get_state().await?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(csv_str.as_bytes());
    let members: Vec<MemberRow> = csv_reader
        .deserialize::<MemberRow>()
        .map(|r| {
            r.map_err(|e| {
                tracing::error!("{e}");
                ServerFnError::new("Something went wrong")
            })
        })
        .collect::<ServerFnResult<Vec<MemberRow>>>()?;

    let mut tx = state.db_pool.begin().await?;

    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO members (id, name, last_name, gender, birthday, mother_id, father_id)",
    );

    query.push_values(members, |mut b, members| {
        b.push_bind(members.id)
            .push_bind(members.name)
            .push_bind(members.last_name)
            .push_bind(members.gender)
            .push_bind(members.birthday)
            .push_bind(members.mother_id)
            .push_bind(members.father_id);
    });

    query
    .push(r#"
            ON CONFLICT(id)
            DO UPDATE SET
            name = EXCLUDED.name, last_name = EXCLUDED.last_name, gender = EXCLUDED.gender,
            birthday = EXCLUDED.birthday, mother_id = EXCLUDED.mother_id, father_id = EXCLUDED.father_id
        "#);

    query.build().execute(&mut *tx).await?;

    sqlx::query!(r#"SELECT setval('members_id_seq', (SELECT MAX(id) FROM members));"#)
        .fetch_optional(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
