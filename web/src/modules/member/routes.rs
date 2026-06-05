use axum::{extract::Extension, response::IntoResponse};

use super::error::MembersError;
use crate::{
    middleware::auth::AuthExtractor,
    modules::{
        member::types::{Gender, MemberRow},
        user::types::UserRole,
    },
    server::AppState,
};

#[axum::debug_handler]
pub async fn export_members(
    Extension(AppState(state)): Extension<AppState>,
    _auth: AuthExtractor<{ UserRole::Admin as u8 }>,
) -> Result<impl IntoResponse, MembersError> {
    let recs = sqlx::query_as!(
        MemberRow,
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
            m.father_id,
            m.mother_id
            FROM members m
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;
    let mut csv_writer = csv::Writer::from_writer(vec![]);
    for rec in recs {
        csv_writer.serialize(rec).map_err(|e| {
            tracing::error!("{e}");
            MembersError::InternalServerError
        })?;
    }
    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/csv"),
        (
            axum::http::header::CONTENT_DISPOSITION,
            r#"attachment; filename="exported-members.csv""#,
        ),
    ];
    csv_writer.flush().map_err(|e| {
        tracing::error!("{e}");
        MembersError::InternalServerError
    })?;
    let data = csv_writer.into_inner().map_err(|e| {
        tracing::error!("{e}");
        MembersError::InternalServerError
    })?;
    Ok((headers, data))
}
