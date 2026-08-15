use dioxus::{fullstack::FileStream, prelude::*};
use indexmap::IndexMap;
use jiff::Zoned;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::middleware::auth::AuthExtractor;
    pub use crate::modules::member::types::MemberRowWithParents;
    pub use crate::modules::member::types::{ChildMember, MemberRow};
    pub use crate::modules::user::types::UserRole;
    pub use crate::server::AppState;
    pub use axum::extract::Extension;
    pub use jiff::tz::TimeZone;
    pub use jiff_sqlx::ToSqlx;
}

use crate::modules::member::types::{EditMember, MemberUnauthorizedResponseFlat};

use super::types::{Gender, MemberResponse, MemberResponseFlat};
#[cfg(feature = "server")]
use server_imports::*;

#[get("/api/v1/members/admin", Extension(state): Extension<AppState>, _user: AuthExtractor<{ UserRole::User as u8 }>)]
pub async fn members() -> anyhow::Result<Option<MemberResponse>> {
    let recs = sqlx::query_as!(
        MemberRowWithParents,
        r#"
            SELECT
                m.id,
                m.name,
                CONCAT_WS(' ', m.name, p2.name, p3.name, p4.name, m.last_name) AS full_name,
                m.gender as "gender: Gender",
                m.birthday as "birthday: jiff_sqlx::Timestamp",
                m.last_name,
                m.image,
                m.image_type,
                m.personal_info,
                m.email,
                mother.id AS mother_id,
                mother.name AS mother_name,
                mother.gender AS "mother_gender: Gender",
                mother.birthday AS "mother_birthday: jiff_sqlx::Timestamp",
                mother.last_name AS mother_last_name,
                father.id AS father_id,
                father.name AS father_name,
                father.gender AS "father_gender: Gender",
                father.birthday AS "father_birthday: jiff_sqlx::Timestamp",
                father.last_name AS father_last_name
            FROM
                members m
            LEFT JOIN
                members mother ON m.mother_id = mother.id
            LEFT JOIN
                members father ON m.father_id = father.id
            LEFT JOIN members p2 ON m.father_id = p2.id
            LEFT JOIN members p3 ON p2.father_id = p3.id
            LEFT JOIN members p4 ON p3.father_id = p4.id;
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
        return Err(anyhow::anyhow!("No root member"));
    };

    let mut root = MemberResponse {
        id: root.id,
        name: root.name.clone(),
        full_name: root.full_name.clone().unwrap_or_else(|| root.name.clone()),
        gender: root.gender,
        birthday: root.birthday.map(|t| t.to_jiff().to_zoned(TimeZone::UTC)),
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

#[get("/api/v1/members/admin/flat", Extension(state): Extension<AppState>, _user: AuthExtractor<{ UserRole::User as u8 }>)]
pub async fn members_flat() -> anyhow::Result<Vec<MemberResponseFlat>> {
    let recs: Vec<MemberRowWithParents> = sqlx::query_as(
        r#"
            SELECT
                m.id,
                m.name,
                CONCAT_WS(' ', m.name, p2.name, p3.name, p4.name, m.last_name) AS full_name,
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
                father.id AS father_id,
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
            LEFT JOIN members p2 ON m.father_id = p2.id
            LEFT JOIN members p3 ON p2.father_id = p3.id
            LEFT JOIN members p4 ON p3.father_id = p4.id;
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .with_context(|| "get members flat")?;

    let all_children: Vec<ChildMember> = sqlx::query_as!(
        ChildMember,
        r#"
        SELECT 
            id,
            name,
            last_name,
            mother_id,
            father_id
        FROM members 
        WHERE mother_id IS NOT NULL OR father_id IS NOT NULL
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .with_context(|| "get members children")?;

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
                name: m.name.clone(),
                full_name: m.full_name.clone().unwrap_or_else(|| m.name),
                gender: m.gender,
                birthday: m.birthday.map(|t| t.to_jiff().to_zoned(TimeZone::UTC)),
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

#[get("/api/v1/members/flat", Extension(state): Extension<AppState>)]
pub async fn members_flat_unauthorized() -> anyhow::Result<Vec<MemberUnauthorizedResponseFlat>> {
    let recs = sqlx::query!(
        r#"
            SELECT
                m.id,
                m.name,
                CONCAT_WS(' ', m.name, p2.name, p3.name, p4.name, m.last_name) AS full_name,
                m.gender AS "gender: Gender",
                m.email,
                m.last_name,
                mother.id AS "mother_id: Option<i64>",
                mother.name AS "mother_name: Option<String>",
                father.id as "father_id: Option<i64>",
                father.name AS "father_name: Option<String>"
            FROM
                members m
            LEFT JOIN
                members mother ON m.mother_id = mother.id
            LEFT JOIN
                members father ON m.father_id = father.id
            LEFT JOIN members p2 ON m.father_id = p2.id
            LEFT JOIN members p3 ON p2.father_id = p3.id
            LEFT JOIN members p4 ON p3.father_id = p4.id
            ORDER BY
            m.id, m.name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .with_context(|| "get members")?;

    let all_children: Vec<ChildMember> = sqlx::query_as!(
        ChildMember,
        r#"
        SELECT 
            id,
            name,
            last_name,
            mother_id,
            father_id
        FROM members 
        WHERE mother_id IS NOT NULL OR father_id IS NOT NULL
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .with_context(|| "get members children")?;

    let members = recs
        .into_iter()
        .map(|m| {
            let children: Vec<ChildMember> = all_children
                .iter()
                .filter(|child| child.mother_id == Some(m.id) || child.father_id == Some(m.id))
                .cloned()
                .collect();
            MemberUnauthorizedResponseFlat {
                id: m.id,
                name: m.name.clone(),
                full_name: m.full_name.clone().unwrap_or_else(|| m.name),
                gender: m.gender,
                last_name: m.last_name,
                father_id: m.father_id,
                mother_id: m.mother_id,
                father_name: m.father_name,
                mother_name: m.mother_name,
                children,
            }
        })
        .collect();

    Ok(members)
}

#[post("/api/v1/members", _admin: AuthExtractor<{ UserRole::Admin as u8 }>, state: Extension<AppState>)]
pub async fn add_member(
    first_name: String,
    last_name: String,
    father_id: Option<i64>,
    mother_id: Option<i64>,
    gender: Gender,
    birthday: Option<Zoned>,
) -> anyhow::Result<i64> {
    if first_name.is_empty() || last_name.is_empty() {
        return Err(anyhow::anyhow!(""));
    }

    let Extension(state) = state;

    let rec = sqlx::query!(
        r#"
            INSERT INTO members (name, last_name, father_id, mother_id, gender, birthday)
            VALUES ($1, $2, $3, $4, $5, $6::text::timestamptz)
            RETURNING id;
        "#,
        first_name,
        last_name,
        father_id,
        mother_id,
        gender as _,
        birthday.map(|z| z.timestamp().to_string()),
    )
    .fetch_one(&state.db_pool)
    .await?;

    Ok(rec.id)
}

#[put("/api/v1/members/{id}", Extension(state): Extension<AppState>)]
pub async fn edit_member(id: i64, edit: EditMember) -> anyhow::Result<()> {
    use crate::modules::types::EditField;

    let mut query = sqlx::QueryBuilder::new("UPDATE members SET ");
    let mut sep = query.separated(", ");
    let mut any = false;

    if let Some(first_name) = edit.name {
        sep.push_unseparated("name = ").push_bind(first_name);

        any = true;
    }

    if let Some(last_name) = edit.last_name {
        sep.push_unseparated("last_name = ").push_bind(last_name);

        any = true;
    }

    if let Some(gender) = edit.gender {
        sep.push_unseparated("gender = ").push_bind(gender);

        any = true;
    }

    match edit.birthday {
        EditField::Changed(birthday) => {
            sep.push_unseparated("birthday = ")
                .push_bind(birthday.timestamp().to_sqlx());

            any = true;
        }
        EditField::Delete => {
            sep.push_unseparated("birthday = NULL");

            any = true;
        }
        EditField::Unchanged => {}
    }

    match edit.father_id {
        EditField::Changed(father_id) => {
            sep.push_unseparated("father_id = ").push_bind(father_id);

            any = true;
        }
        EditField::Delete => {
            sep.push_unseparated("father_id = NULL");

            any = true;
        }
        EditField::Unchanged => {}
    }

    match edit.mother_id {
        EditField::Changed(mother_id) => {
            sep.push_unseparated("mother_id = ").push_bind(mother_id);

            any = true;
        }
        EditField::Delete => {
            sep.push_unseparated("mother_id = NULL");

            any = true;
        }
        EditField::Unchanged => {}
    }

    match &edit.personal_info {
        EditField::Changed(personal_info) => {
            sep.push_unseparated("personal_info = ")
                .push_bind(sqlx::types::Json::from(personal_info));

            any = true;
        }
        EditField::Delete => {
            sep.push_unseparated("personal_info = NULL");

            any = true;
        }
        EditField::Unchanged => {}
    }

    if any {
        query.push(" WHERE id = ").push_bind(id);

        query.build().execute(&state.db_pool).await?;
    }

    Ok(())
}

#[put("/api/v1/members/{id}/image", Extension(state): Extension<AppState>)]
pub async fn edit_member_image(id: i64, mut image: FileStream) -> anyhow::Result<()> {
    use futures::StreamExt;
    let mut image_bytes = Vec::with_capacity(image.size().unwrap_or_default() as usize);
    while let Some(Ok(chunk)) = image.next().await {
        image_bytes.extend(chunk.as_ref());
    }
    sqlx::query!(
        r#"
            UPDATE members
            SET image = $1
            WHERE id = $2;
        "#,
        image_bytes,
        id,
    )
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

#[delete("/api/v1/members/{id}/image", Extension(state): Extension<AppState>)]
pub async fn delete_member_image(id: i64) -> anyhow::Result<()> {
    Ok(())
}

#[delete("/api/v1/members/{id}", Extension(state): Extension<AppState>)]
pub async fn delete_member(id: i64) -> anyhow::Result<()> {
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

#[post("/api/v1/members/csv", Extension(state): Extension<AppState>)]
pub async fn upload_members_csv(csv_str: String) -> anyhow::Result<()> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(csv_str.as_bytes());
    let members: Vec<MemberRow> = csv_reader
        .deserialize::<MemberRow>()
        .map(|r| {
            r.map_err(|e| {
                tracing::error!("{e}");
                anyhow::anyhow!("Something went wrong")
            })
        })
        .collect::<anyhow::Result<Vec<MemberRow>>>()?;

    let mut tx = state.db_pool.begin().await?;

    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO members (id, name, last_name, gender, birthday, mother_id, father_id)",
    );

    query.push_values(members, |mut b, members| {
        b.push_bind(members.id)
            .push_bind(members.name)
            .push_bind(members.last_name)
            .push_bind(members.gender)
            .push_bind(members.birthday.map(|z| z.timestamp().to_sqlx()))
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
