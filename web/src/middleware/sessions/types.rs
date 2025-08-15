use chrono::DateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub expires_at: DateTime<chrono::Utc>,
    pub user_id: Uuid,
}

pub struct CreateSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub expires_at: DateTime<chrono::Utc>,
}
