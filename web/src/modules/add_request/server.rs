use anyhow::anyhow;
use dioxus::prelude::*;
use garde::Validate;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::modules::add_request::types::{
    RequestChildData, RequestData, RequestStatus, RequestedMember,
};
use crate::modules::member::types::Gender;
use crate::modules::user::types::UserRole;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::AuthExtractor;
    pub use crate::modules::add_request::types::{
        RequestedMemberBrief, RequestedMemberRowWithParents,
    };
    pub use crate::modules::settings::server::add_request_rules;
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
    pub use jiff::tz::TimeZone;
}

#[cfg(feature = "server")]
use server_imports::*;

#[post("/api/v1/members/request", Extension(state): Extension<AppState>)]
pub async fn add_request(request_data: RequestData) -> Result<(), anyhow::Error> {
    let rules = add_request_rules(&state.db_pool).await?;

    request_data.validate_with(&rules)?;

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
        children,
    } = request_data
    else {
        return Err(anyhow!("Something went wrong"));
    };

    let Ok(info) = serde_json::value::to_value(info) else {
        return Err(anyhow!("Something went wrong"));
    };

    let mut tx = state.db_pool.begin().await?;

    let request = sqlx::query!(
        r#"
            INSERT INTO member_add_requests (id, name, gender, birthday, last_name, father_id, mother_id, image, image_type, personal_info, submitted_at)
            VALUES ($1, $2, $3, $4::text::timestamptz, $5, $6, $7, $8, $9, $10, now())
            RETURNING id;
        "#,
        uuid::Uuid::new_v4(), name, gender as _, birthday.map(|z| z.timestamp().to_string()), last_name, father_id, mother_id,
        image, image_type, info,
    )
    .fetch_one(&mut *tx)
    .await?;

    let mother_request_id = matches!(gender, Gender::Female).then_some(request.id);
    let father_request_id = matches!(gender, Gender::Male).then_some(request.id);

    for child in children {
        let RequestChildData {
            name: Some(name),
            gender: Some(gender),
            birthday,
            info,
            image,
            image_type,
        } = child
        else {
            return Err(anyhow!("Something went wrong"));
        };

        let Ok(info) = serde_json::value::to_value(info) else {
            return Err(anyhow!("Something went wrong"));
        };

        sqlx::query!(
            r#"
                INSERT INTO member_add_requests (id, name, gender, birthday, last_name, image, image_type, personal_info, submitted_at, mother_request_id, father_request_id)
                VALUES ($1, $2, $3, $4::text::timestamptz, $5, $6, $7, $8, now(), $9, $10)
            "#,
            uuid::Uuid::new_v4(), name, gender as _, birthday.map(|z| z.timestamp().to_string()), last_name,
            image, image_type, info,
            mother_request_id, father_request_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

#[get("/api/v1/members/request", _admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn member_requests() -> Result<Vec<RequestedMember>, anyhow::Error> {
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
            CONCAT_WS(' ', father.name, grandfather.name, greatfather.name, greatgrandfather.name, father.last_name) AS father_name,
            father.gender AS father_gender,
            father.birthday AS father_birthday,
            father.last_name AS father_last_name
        FROM
            member_add_requests m
        LEFT JOIN
            members mother ON m.mother_id = mother.id
        LEFT JOIN
            members father ON m.father_id = father.id
        LEFT JOIN
            members grandfather ON father.father_id = grandfather.id
        LEFT JOIN
            members greatfather ON grandfather.father_id = greatfather.id
        LEFT JOIN
            members greatgrandfather ON greatfather.father_id = greatgrandfather.id
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
            birthday: m.birthday.map(|t| t.to_jiff().to_zoned(TimeZone::UTC)),
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

#[put("/api/v1/members/request/approve", admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn approve_request(request_id: Uuid) -> Result<(), anyhow::Error> {
    use sqlx::types::Json;

    let mut tx = state.db_pool.begin().await?;

    let member_request = sqlx::query_as!(
        RequestedMemberBrief,
        r#"
            UPDATE member_add_requests request
            SET reviewed_at = now(), reviewed_by = $1, status = 'approved'
            WHERE request.id = $2
            RETURNING id, name, gender as "gender: Gender", birthday as "birthday: jiff_sqlx::Timestamp", last_name, image, status as "status: RequestStatus",
                image_type, mother_id, father_id, personal_info as "personal_info: Json<IndexMap<String, String>>";
        "#,
        admin.current_user.id,
        request_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let child_requests = sqlx::query_as!(
        RequestedMemberBrief,
        r#"
            UPDATE member_add_requests request
            SET reviewed_at = now(), reviewed_by = $1, status = 'approved'
            WHERE request.mother_request_id = $2 OR request.father_request_id = $2
            RETURNING id, name, gender as "gender: Gender", birthday as "birthday: jiff_sqlx::Timestamp", last_name, image, status as "status: RequestStatus",
                image_type, mother_id, father_id, personal_info as "personal_info: Json<IndexMap<String, String>>";
        "#,
        admin.current_user.id,
        request_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    let Some(member_request) = member_request else {
        return Err(anyhow!("Bad Request"));
    };

    let Ok(info) = serde_json::value::to_value(member_request.personal_info) else {
        return Err(anyhow!("Something went wrong"));
    };

    let new_member = sqlx::query!(
        r#"
            INSERT INTO members (name, gender, birthday, last_name, father_id, mother_id, personal_info, image, image_type)
            VALUES ($1, $2, $3::text::timestamptz, $4, $5, $6, $7, $8, $9)
            RETURNING id;
        "#,
        member_request.name,
        member_request.gender as _,
        member_request.birthday.map(|t| t.to_jiff().to_string()),
        member_request.last_name,
        member_request.father_id,
        member_request.mother_id,
        info,
        member_request.image,
        member_request.image_type,
    )
    .fetch_one(&mut *tx)
    .await?;

    for child_request in child_requests {
        let Ok(info) = serde_json::value::to_value(child_request.personal_info) else {
            return Err(anyhow!("Something went wrong"));
        };

        sqlx::query!(
            r#"
                INSERT INTO members (name, gender, birthday, last_name, father_id, mother_id, personal_info, image, image_type)
                VALUES ($1, $2, $3::text::timestamptz, $4, $5, $6, $7, $8, $9);
            "#,
            child_request.name,
            child_request.gender as _,
            child_request.birthday.map(|t| t.to_jiff().to_string()),
            child_request.last_name,
            (member_request.gender == Gender::Male).then_some(new_member.id),
            (member_request.gender == Gender::Female).then_some(new_member.id),
            info,
            child_request.image,
            child_request.image_type,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

#[put("/api/v1/members/request/disapprove", admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn disapprove_request(request_id: Uuid) -> Result<(), anyhow::Error> {
    let mut tx = state.db_pool.begin().await?;

    sqlx::query!(
        r#"
            UPDATE member_add_requests
            SET status = 'disapproved',
                reviewed_at = NOW(),
                reviewed_by = $1
            WHERE id = $2
            OR mother_request_id = $2
            OR father_request_id = $2;
        "#,
        admin.current_user.id,
        request_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
