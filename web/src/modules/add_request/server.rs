use chrono::Utc;
use dioxus::prelude::*;
use garde::Validate;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::modules::add_request::types::{
    RequestData, RequestStatus, RequestedMember, RequestedMemberBrief,
    RequestedMemberRowWithParents,
};
use crate::modules::member::types::Gender;

#[server]
pub async fn add_request(request_data: RequestData) -> ServerFnResult<()> {
    use crate::server::get_state;

    request_data.validate()?;

    let RequestData {
        name: Some(name),
        last_name: Some(last_name),
        gender: Some(gender),
        birthday,
        father_id,
        mother_id,
        info,
        image,
        image_type,
    } = request_data
    else {
        return Err(ServerFnError::new("Something went wrong"));
    };

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
        image, image_type, info, Utc::now(),
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[server]
pub async fn member_requests() -> ServerFnResult<Vec<RequestedMember>> {
    use crate::modules::admin::server::get_admin;
    use crate::server::get_state;

    let _admin = get_admin().await?;

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

    let requested_members: Vec<RequestedMember> = recs
        .into_iter()
        .map(|m| RequestedMember {
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

#[server]
pub async fn approve_request(request_id: Uuid) -> ServerFnResult<()> {
    use crate::modules::admin::server::get_admin;
    use crate::server::get_state;

    use sqlx::types::Json;

    let admin = get_admin().await?;

    let state = get_state().await?;

    let mut tx = state.db_pool.begin().await?;

    let requested_member_info = sqlx::query_as!(
        RequestedMemberBrief,
        r#"
            UPDATE member_add_requests request
            SET reviewed_at = $1, reviewed_by = $2, status = 'approved'
            WHERE request.id = $3
            RETURNING id, name, gender as "gender: Gender", birthday, last_name, image, status as "status: RequestStatus",
                image_type, mother_id, father_id, personal_info as "personal_info: Json<IndexMap<String, String>>";
        "#,
        Utc::now(),
        admin.id,
        request_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(member_info) = requested_member_info else {
        return Err(ServerFnError::new("Bad Request"));
    };

    sqlx::query!(
        r#"
            INSERT INTO members (name, last_name, gender, birthday)
            VALUES ($1, $2, $3, $4);
        "#,
        member_info.name,
        member_info.last_name,
        member_info.gender as _,
        member_info.birthday,
    )
    .execute(&state.db_pool)
    .await?;

    tx.commit().await?;

    Ok(())
}

#[server]
pub async fn disapprove_request(request_id: Uuid) -> ServerFnResult<()> {
    use crate::modules::admin::server::get_admin;
    use crate::server::get_state;

    let admin = get_admin().await?;

    let state = get_state().await?;

    let mut tx = state.db_pool.begin().await?;

    sqlx::query!(
        r#"
            UPDATE member_add_requests request
            SET reviewed_at = $1, reviewed_by = $2, status = 'disapproved'
            WHERE request.id = $3;
        "#,
        Utc::now(),
        admin.id,
        request_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
