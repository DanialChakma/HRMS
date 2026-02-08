use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attendance {
    pub id: u64,
    pub employee_id: u64,
    pub date: NaiveDate,
    pub check_in: Option<NaiveTime>,
    pub check_out: Option<NaiveTime>,
}
