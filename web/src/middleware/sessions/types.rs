use jiff_sqlx::Timestamp;
use uuid::Uuid;
#[derive(sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub user_id: Uuid,
}
pub struct CreateSession {
    pub id: Uuid,
    pub user_id: Uuid,
}
