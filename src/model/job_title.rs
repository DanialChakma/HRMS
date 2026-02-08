use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobTitle {
    pub id: u64,
    pub title: String,
}
