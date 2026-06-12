use dioxus::prelude::*;

use super::types::MemberSearch;

#[cfg(feature = "server")]
mod server_imports {
    pub use crate::modules::member::types::Gender;
    pub use crate::server::AppState;
    pub use axum::Extension;
}

#[cfg(feature = "server")]
use server_imports::*;

#[get("/api/v1/member/search?q", Extension(state): Extension<AppState>)]
pub async fn search_member(q: String) -> anyhow::Result<Vec<MemberSearch>> {
    Ok(sqlx::query_as!(
        MemberSearch,
        r#"
            SELECT
            p1.id,
            p1.name,
            CONCAT_WS(' ', p1.name, p2.name, p3.name, p4.name, p1.last_name) AS full_name,
            p1.last_name,
            p1.gender as "gender: Gender",
            p1.birthday,
            p2.name AS "father: Option<String>",
            p3.name AS "grandfather: Option<String>",
            p4.name AS "great_grandfather: Option<String>"
            FROM members p1
            LEFT JOIN members p2 ON p1.father_id = p2.id
            LEFT JOIN members p3 ON p2.father_id = p3.id
            LEFT JOIN members p4 ON p3.father_id = p4.id
            WHERE
            p1.name % $1
            OR p1.last_name % $1
            OR p2.name % $1
            OR p3.name % $1
            OR p4.name % $1
            ORDER BY (
                similarity(CONCAT_WS(' ', p1.name, p2.name, p3.name, p4.name, p1.last_name), $1) * 0.7
                    - ABS(
                        LENGTH(CONCAT_WS(' ', p1.name, p2.name, p3.name, p4.name, p1.last_name)) - LENGTH($1)
                    )::float / GREATEST(LENGTH(CONCAT_WS(' ', p1.name, p2.name, p3.name, p4.name, p1.last_name)), LENGTH($1)) * 0.3
            )
            DESC
            LIMIT 20;
        "#,
        q
    )
    .fetch_all(&state.db_pool)
    .await?)
}

#[get("/api/v1/member/by-name/{name}", Extension(state): Extension<AppState>)]
pub async fn get_member_by_name(name: String) -> anyhow::Result<Option<MemberSearch>> {
    Ok(sqlx::query_as!(
        MemberSearch,
        r#"
            SELECT
            p1.id,
            p1.name,
            CONCAT_WS(' ', p1.name, p2.name, p3.name, p4.name, p1.last_name) AS full_name,
            p1.last_name,
            p1.gender as "gender: Gender",
            p1.birthday,
            p2.name AS father,
            p3.name AS grandfather,
            p4.name AS great_grandfather
            FROM members p1
            LEFT JOIN members p2 ON p1.father_id = p2.id
            LEFT JOIN members p3 ON p2.father_id = p3.id
            LEFT JOIN members p4 ON p3.father_id = p4.id
            WHERE
            CONCAT_WS(' ', p1.name, p2.name, p3.name, p4.name, p1.last_name) = $1
        "#,
        name
    )
    .fetch_optional(&state.db_pool)
    .await?)
}
