use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payroll {
    pub id: u64,
    pub employee_id: u64,
    pub month: NaiveDate,
    pub base_salary: f64,
    pub bonus: f64,
    pub deductions: f64,
    pub net_salary: f64,
}
