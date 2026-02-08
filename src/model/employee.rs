use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, openapi::schema};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[schema(
    example = json!({
        "id": 1,
        "employee_code": "EMP-001",
        "first_name": "John",
        "last_name": "Doe",
        "email": "john.doe@company.com",
        "phone": "+8801712345678",
        "department_id": 10,
        "job_title_id": 3,
        "hire_date": "2024-01-01",
        "status": "active"
    })
)]
pub struct Employee {
    #[schema(example = 1)]
    pub id: u64,

    #[schema(example = "EMP-001")]
    pub employee_code: String,

    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john.doe@company.com")]
    pub email: String,

    #[schema(example = "+8801712345678", nullable = true)]
    pub phone: Option<String>,

    #[schema(example = 10)]
    pub department_id: u64,

    #[schema(example = 3)]
    pub job_title_id: u64,

    #[schema(
        example = "2024-01-01",
        value_type = String,
        format = "date"
    )]
    pub hire_date: NaiveDate,

    #[schema(example = "active")]
    pub status: String,
}

