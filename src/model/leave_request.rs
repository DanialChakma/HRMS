use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaveRequest {
    pub id: u64,
    pub employee_id: u64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub leave_type: String,
    pub status: String,
}
