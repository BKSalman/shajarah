use anyhow::anyhow;
use dioxus::prelude::*;
use garde::Validate;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::modules::add_request::types::{
    RequestChildData, RequestData, RequestSpouseData, RequestStatus, RequestedMember, SpouseKind,
};
use crate::modules::member::types::{Gender, MarriageStatus};
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
        spouse,
        children,
    } = request_data
    else {
        return Err(anyhow!("Something went wrong"));
    };

    let Ok(info) = serde_json::value::to_value(info) else {
        return Err(anyhow!("Something went wrong"));
    };

    // The client picks the spouse's gender, so re-check what the schema can't:
    // a marriage is between opposite genders, and a parent is not a spouse.
    if let Some(spouse) = &spouse {
        match spouse.kind {
            SpouseKind::Existing => {
                let spouse_id = spouse.member_id;

                if spouse_id.is_some() && (spouse_id == father_id || spouse_id == mother_id) {
                    return Err(anyhow!("لا يمكن أن يكون الوالد أو الوالدة زوجاً"));
                }
            }
            SpouseKind::New => {
                if spouse.gender == Some(gender) {
                    return Err(anyhow!("لا يمكن ربط شخصين من نفس الجنس"));
                }
            }
        }
    }

    let mut tx = state.db_pool.begin().await?;

    // Only the existing-member case puts the link on the main row; a new spouse
    // gets its own row pointing back here, and carries the state itself.
    let (spouse_id, marriage_status) = match &spouse {
        Some(spouse) if spouse.kind == SpouseKind::Existing => (spouse.member_id, spouse.status),
        _ => (None, None),
    };

    let request = sqlx::query!(
        r#"
            INSERT INTO member_add_requests (id, name, gender, birthday, last_name, father_id, mother_id, image, image_type, personal_info, submitted_at, spouse_id, marriage_status)
            VALUES ($1, $2, $3, $4::text::timestamptz, $5, $6, $7, $8, $9, $10, now(), $11, $12)
            RETURNING id;
        "#,
        uuid::Uuid::new_v4(), name, gender as _, birthday.map(|z| z.timestamp().to_string()), last_name, father_id, mother_id,
        image, image_type, info, spouse_id, marriage_status as _,
    )
    .fetch_one(&mut *tx)
    .await?;

    let spouse_request_id = match spouse {
        Some(
            spouse @ RequestSpouseData {
                kind: SpouseKind::New,
                ..
            },
        ) => {
            let RequestSpouseData {
                name: Some(spouse_name),
                last_name: Some(spouse_last_name),
                gender: Some(spouse_gender),
                birthday,
                info,
                image,
                image_type,
                status,
                ..
            } = spouse
            else {
                return Err(anyhow!("Something went wrong"));
            };

            let Ok(info) = serde_json::value::to_value(info) else {
                return Err(anyhow!("Something went wrong"));
            };

            let spouse_request = sqlx::query!(
                r#"
                    INSERT INTO member_add_requests (id, name, gender, birthday, last_name, image, image_type, personal_info, submitted_at, spouse_of_request_id, marriage_status)
                    VALUES ($1, $2, $3, $4::text::timestamptz, $5, $6, $7, $8, now(), $9, $10)
                    RETURNING id;
                "#,
                uuid::Uuid::new_v4(), spouse_name, spouse_gender as _, birthday.map(|z| z.timestamp().to_string()),
                spouse_last_name, image, image_type, info, request.id,
                status.unwrap_or_default() as _,
            )
            .fetch_one(&mut *tx)
            .await?;

            Some(spouse_request.id)
        }
        _ => None,
    };

    // Which parent slot the requested member fills; the spouse fills the other.
    let main_gender = gender;
    let mother_request_id = matches!(main_gender, Gender::Female).then_some(request.id);
    let father_request_id = matches!(main_gender, Gender::Male).then_some(request.id);

    for child in children {
        let RequestChildData {
            name: Some(name),
            gender: Some(gender),
            birthday,
            info,
            image,
            image_type,
            from_spouse,
        } = child
        else {
            return Err(anyhow!("Something went wrong"));
        };

        let Ok(info) = serde_json::value::to_value(info) else {
            return Err(anyhow!("Something went wrong"));
        };

        // A child of the couple fills the other parent slot too: a request id
        // when the spouse is new, a member id when they were picked from the
        // tree. Without a spouse this stays exactly as it was before.
        let from_spouse = from_spouse && (spouse_id.is_some() || spouse_request_id.is_some());

        let (child_mother_request_id, child_father_request_id, child_mother_id, child_father_id) =
            match (from_spouse, main_gender) {
                (false, _) => (mother_request_id, father_request_id, None, None),
                // the member is the father, so the spouse is the mother
                (true, Gender::Male) => (spouse_request_id, father_request_id, spouse_id, None),
                (true, Gender::Female) => (mother_request_id, spouse_request_id, None, spouse_id),
            };

        sqlx::query!(
            r#"
                INSERT INTO member_add_requests (id, name, gender, birthday, last_name, image, image_type, personal_info, submitted_at, mother_request_id, father_request_id, mother_id, father_id)
                VALUES ($1, $2, $3, $4::text::timestamptz, $5, $6, $7, $8, now(), $9, $10, $11, $12)
            "#,
            uuid::Uuid::new_v4(), name, gender as _, birthday.map(|z| z.timestamp().to_string()), last_name,
            image, image_type, info,
            child_mother_request_id, child_father_request_id, child_mother_id, child_father_id,
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
            m.mother_request_id,
            m.father_request_id,
            m.spouse_id,
            m.spouse_of_request_id,
            m.marriage_status,
            CONCAT_WS(' ', spouse.name, spouse.last_name) AS spouse_name,
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
            members spouse ON m.spouse_id = spouse.id
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
            mother_request_id: m.mother_request_id,
            father_request_id: m.father_request_id,
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
            spouse_id: m.spouse_id,
            spouse_of_request_id: m.spouse_of_request_id,
            spouse_name: m.spouse_name,
            marriage_status: m.marriage_status,
        })
        .collect();

    Ok(requested_members)
}

#[put("/api/v1/members/request/approve", admin: AuthExtractor<{ UserRole::Admin as u8 }>, Extension(state): Extension<AppState>)]
pub async fn approve_request(request_id: Uuid) -> Result<(), anyhow::Error> {
    use crate::modules::member::types::spouse_columns;
    use sqlx::types::Json;

    let mut tx = state.db_pool.begin().await?;

    let member_request = sqlx::query_as!(
        RequestedMemberBrief,
        r#"
            UPDATE member_add_requests request
            SET reviewed_at = now(), reviewed_by = $1, status = 'approved'
            WHERE request.id = $2
            RETURNING id, name, gender as "gender: Gender", birthday as "birthday: jiff_sqlx::Timestamp", last_name, image, status as "status: RequestStatus",
                image_type, mother_id, father_id, personal_info as "personal_info: Json<IndexMap<String, String>>",
                mother_request_id, father_request_id, spouse_id, spouse_of_request_id,
                marriage_status as "marriage_status: MarriageStatus";
        "#,
        admin.current_user.id,
        request_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    // The spouse submitted as a new outside person, if there was one.
    let spouse_request = sqlx::query_as!(
        RequestedMemberBrief,
        r#"
            UPDATE member_add_requests request
            SET reviewed_at = now(), reviewed_by = $1, status = 'approved'
            WHERE request.spouse_of_request_id = $2
            RETURNING id, name, gender as "gender: Gender", birthday as "birthday: jiff_sqlx::Timestamp", last_name, image, status as "status: RequestStatus",
                image_type, mother_id, father_id, personal_info as "personal_info: Json<IndexMap<String, String>>",
                mother_request_id, father_request_id, spouse_id, spouse_of_request_id,
                marriage_status as "marriage_status: MarriageStatus";
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
                image_type, mother_id, father_id, personal_info as "personal_info: Json<IndexMap<String, String>>",
                mother_request_id, father_request_id, spouse_id, spouse_of_request_id,
                marriage_status as "marriage_status: MarriageStatus";
        "#,
        admin.current_user.id,
        request_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    let Some(member_request) = member_request else {
        return Err(anyhow!("Bad Request"));
    };

    let Ok(info) = serde_json::value::to_value(member_request.personal_info.clone()) else {
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

    // A new outside spouse becomes a member with no parent links; a spouse
    // picked from the tree is already one.
    let new_spouse = match &spouse_request {
        Some(spouse_request) => {
            let Ok(info) = serde_json::value::to_value(spouse_request.personal_info.clone()) else {
                return Err(anyhow!("Something went wrong"));
            };

            let spouse = sqlx::query!(
                r#"
                    INSERT INTO members (name, gender, birthday, last_name, father_id, mother_id, personal_info, image, image_type)
                    VALUES ($1, $2, $3::text::timestamptz, $4, NULL, NULL, $5, $6, $7)
                    RETURNING id;
                "#,
                spouse_request.name,
                spouse_request.gender as _,
                spouse_request.birthday.map(|t| t.to_jiff().to_string()),
                spouse_request.last_name,
                info,
                spouse_request.image,
                spouse_request.image_type,
            )
            .fetch_one(&mut *tx)
            .await?;

            Some((spouse.id, spouse_request.gender))
        }
        None => match member_request.spouse_id {
            Some(spouse_id) => {
                let spouse = sqlx::query!(
                    r#"SELECT id, gender as "gender: Gender" FROM members WHERE id = $1"#,
                    spouse_id,
                )
                .fetch_optional(&mut *tx)
                .await?;

                // The picked member may have been deleted while the request sat
                // pending, which nulls spouse_id — approving the rest still works.
                spouse.map(|spouse| (spouse.id, spouse.gender))
            }
            None => None,
        },
    };

    if let Some((spouse_id, spouse_gender)) = new_spouse {
        let Some((husband_id, wife_id)) = spouse_columns(
            (new_member.id, member_request.gender),
            (spouse_id, spouse_gender),
        ) else {
            return Err(anyhow!("لا يمكن ربط شخصين من نفس الجنس"));
        };

        let status = member_request
            .marriage_status
            .or_else(|| spouse_request.as_ref().and_then(|s| s.marriage_status))
            .unwrap_or_default();

        sqlx::query!(
            r#"
                INSERT INTO marriages (husband_id, wife_id, status)
                VALUES ($1, $2, $3)
                ON CONFLICT (husband_id, wife_id)
                DO UPDATE SET status = EXCLUDED.status;
            "#,
            husband_id,
            wife_id,
            status as _,
        )
        .execute(&mut *tx)
        .await?;
    }

    for child_request in child_requests {
        let Ok(info) = serde_json::value::to_value(child_request.personal_info) else {
            return Err(anyhow!("Something went wrong"));
        };

        // Each child points at the request rows of its parents; map those to the
        // members just created. A child with no spouse involved resolves to the
        // requested member alone, exactly as before.
        let resolve = |request_id: Option<Uuid>, member_id: Option<i64>| -> Option<i64> {
            match request_id {
                Some(id) if id == member_request.id => Some(new_member.id),
                Some(id) if Some(id) == spouse_request.as_ref().map(|s| s.id) => {
                    new_spouse.map(|(id, _)| id)
                }
                Some(_) => None,
                None => member_id,
            }
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
            resolve(child_request.father_request_id, child_request.father_id),
            resolve(child_request.mother_request_id, child_request.mother_id),
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
            OR father_request_id = $2
            OR spouse_of_request_id = $2;
        "#,
        admin.current_user.id,
        request_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
