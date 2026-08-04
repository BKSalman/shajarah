use jiff::Zoned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Server-side row read from the database (timestamps as sqlx wrappers).
#[cfg(feature = "server")]
#[derive(Clone, sqlx::prelude::FromRow)]
pub struct InviteRow {
    pub id: i64,
    pub token_hash: String,
    pub created_by: Uuid,
    pub expires_at: Option<jiff_sqlx::Timestamp>,
    pub used_at: Option<jiff_sqlx::Timestamp>,
    pub accepted_by: Option<Uuid>,
    pub created_at: jiff_sqlx::Timestamp,
}

/// Wire representation sent to the client.
#[derive(Clone, Serialize, Deserialize)]
pub struct Invite {
    pub id: i64,
    pub token_hash: String,
    pub created_by: Uuid,
    pub expires_at: Option<Zoned>,
    pub used_at: Option<Zoned>,
    pub accepted_by: Option<Uuid>,
    pub created_at: Zoned,
}
