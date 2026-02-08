use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password: String,
    pub role_id: u8,
    pub employee_id: Option<u64>,
    pub is_active: bool,
}
