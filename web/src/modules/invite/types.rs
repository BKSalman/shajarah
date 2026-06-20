use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "server", derive(sqlx::prelude::FromRow))]
#[derive(Clone, Serialize, Deserialize)]
pub struct InviteRow {
    pub id: i64,
    pub token_hash: String,
    pub created_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub used_at: Option<DateTime<Utc>>,
    pub accepted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
