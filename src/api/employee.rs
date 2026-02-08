use crate::{
    model::employee::Employee,
    utils::db_utils::{build_update_sql, execute_update},
};
use actix_web::{HttpResponse, Responder, error::ErrorInternalServerError, web};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::MySqlPool;
use tracing::{debug, error};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateEmployee {
    #[schema(example = "3000", value_type = String)]
    pub employee_code: String,
    #[schema(example = "first name", value_type = String)]
    pub first_name: String,
    #[schema(example = "last name", value_type = String)]
    pub last_name: String,
    #[schema(example = "john@email.com", format = "email", value_type = String)]
    pub email: String,
    #[schema(example = 1, value_type = u64 )]
    pub department_id: u64,
    #[schema(example = 2, value_type = u64 )]
    pub job_title_id: u64,
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    pub hire_date: chrono::NaiveDate,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EmployeeQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub department_id: Option<u64>,
    pub job_title_id: Option<u64>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EmployeeListResponse {
    #[schema(
    example = json!([{
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
    }])
)]
    pub data: Vec<Employee>,
    #[schema(example = 1)]
    pub page: u32,
    #[schema(example = 5)]
    pub per_page: u32,
    #[schema(example = 10)]
    pub total: i64,
}

#[derive(Serialize, ToSchema)]
pub struct EmployeeResponse {
    pub id: u64,
    pub employee_code: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
    pub department_id: u64,
    pub job_title_id: u64,
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    pub hire_date: chrono::NaiveDate,
    pub status: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateEmployee {
    pub employee_code: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub department_id: Option<u64>,
    pub job_title_id: Option<u64>,
    pub status: Option<String>,
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    pub hire_date: Option<NaiveDate>,
}

/// Create Employee
#[utoipa::path(
    post,
    path = "/api/v1/employees",
    request_body = CreateEmployee,
    responses(
        (status = 200, description = "Employee created successfully", body = Object, example = json!({
            "message": "User registered successfully"
        })),
        (status = 500, description = "Internal server error", body = Object, example = json!({
            "message": "Something went wrong, Contact with system admin"
        }))
    ),
    tag = "Employee",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_employee(
    // auth: AuthUser,
    pool: web::Data<MySqlPool>,
    payload: web::Json<CreateEmployee>,
) -> impl Responder {
    // auth.require_hr_or_admin()?;

    let result = sqlx::query!(
        r#"
        INSERT INTO employees
        (employee_code, first_name, last_name, email, department_id, job_title_id, hire_date)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        payload.employee_code,
        payload.first_name,
        payload.last_name,
        payload.email,
        payload.department_id,
        payload.job_title_id,
        payload.hire_date,
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "User registered successfully"
        })),
        Err(e) => {
            error!(error = %e, "Failed to Create Employee");
            HttpResponse::InternalServerError().json(json!({
                "message":"Something went wrong, Contact with system admin"
            }))
        }
    }
}

// -------------------- Handler --------------------

#[utoipa::path(
    get,
    path = "/api/v1/employees",
    params(
        ("page",  Query, description = "Page number"),
        ("per_page", Query, description = "Items per page"),
        ("department_id", Query, description = "Filter by department"),
        ("job_title_id", Query, description = "Filter by job title"),
        ("status", Query, description = "Filter by status"),
        ("search", Query, description = "Search by name or email")
    ),
    responses(
        (status = 200, description = "Paginated employee list", body = EmployeeListResponse)
    ),
    tag = "Employee",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_employees(
    // auth: AuthUser, // optional auth
    pool: web::Data<MySqlPool>,
    query: web::Query<EmployeeQuery>,
) -> actix_web::Result<impl Responder> {
    // Example: only HR/Admin allowed
    // auth.require_hr_or_admin()?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // ---------- build WHERE clause dynamically ----------
    let mut conditions = Vec::new();
    let mut bindings: Vec<sqlx::types::JsonValue> = Vec::new();

    if let Some(department_id) = query.department_id {
        conditions.push("department_id = ?");
        bindings.push(department_id.into());
    }

    if let Some(job_title_id) = query.job_title_id {
        conditions.push("job_title_id = ?");
        bindings.push(job_title_id.into());
    }

    if let Some(status) = &query.status {
        conditions.push("status = ?");
        bindings.push(status.clone().into());
    }

    if let Some(search) = &query.search {
        conditions.push("(first_name LIKE ? OR last_name LIKE ? OR email LIKE ?)");
        let like = format!("%{}%", search);
        bindings.push(like.clone().into());
        bindings.push(like.clone().into());
        bindings.push(like.into());
    }

    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // ---------- total count ----------
    let count_sql = format!("SELECT COUNT(*) as total FROM employees {}", where_clause);
    debug!(sql = %count_sql, bindings = ?bindings, "Counting employees");

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &bindings {
        count_query = count_query.bind(b);
    }

    let total = count_query.fetch_one(pool.get_ref()).await.map_err(|e| {
        error!(error = %e, sql = %count_sql, "Failed to count employees");
        ErrorInternalServerError("Database error")
    })?;

    // ---------- data query ----------
    let data_sql = format!(
        "SELECT * FROM employees {} ORDER BY id DESC LIMIT ? OFFSET ?",
        where_clause
    );
    debug!(sql = %data_sql, bindings = ?bindings, page, per_page, offset, "Fetching employees");

    let mut data_query = sqlx::query_as::<_, Employee>(&data_sql);
    for b in &bindings {
        data_query = data_query.bind(b);
    }
    data_query = data_query.bind(per_page as i64).bind(offset as i64);

    let employees = data_query.fetch_all(pool.get_ref()).await.map_err(|e| {
        error!(error = %e, sql = %data_sql, "Failed to fetch employees");
        ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(EmployeeListResponse {
        data: employees,
        page,
        per_page,
        total,
    }))
}

/// Update Employee
#[utoipa::path(
    put,
    path = "/api/v1/employees/{employee_id}",
    params(
        ("employee_id", Path, description = "Employee ID")
    ),
    request_body = UpdateEmployee,
    responses(
        (status = 200, description = "Employee updated successfully", body = Object, example = json!({
            "message": "Employee updated successfully"
        })),
        (status = 404, description = "Employee not found", body = Object, example = json!({
            "message": "Employee not found"
        })),
        (status = 500, description = "Internal server error")
    ),
    tag = "Employee",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_employee(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    body: web::Json<Value>,
) -> actix_web::Result<impl Responder> {
    let employee_id = path.into_inner();

    let update = build_update_sql("employees", &body, "id", employee_id)?;

    let affected = execute_update(pool.get_ref(), update)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if affected == 0 {
        return Ok(HttpResponse::NotFound().body("Employee not found"));
    }

    Ok(HttpResponse::Ok().body("Employee updated successfully"))
}

/// Delete Employee
#[utoipa::path(
    delete,
    path = "/api/v1/employees/{employee_id}",
    params(
        ("employee_id", Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted", body = Object, example = json!({
            "message": "Successfully deleted"
        })),
        (status = 404, description = "Employee not found", body = Object, example = json!({
            "message": "Employee not found"
        })),
        (status = 500, description = "Internal server error", body = Object)
    ),
    tag = "Employee",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_employee(
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
) -> actix_web::Result<impl Responder> {
    let employee_id = path.into_inner();

    let result = sqlx::query!(r#"DELETE FROM employees WHERE id = ?"#, employee_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(res) => {
            // Optional: check if a row was actually deleted
            if res.rows_affected() == 0 {
                return Ok(HttpResponse::NotFound().json(json!({
                    "message": "Employee not found"
                })));
            }

            Ok(HttpResponse::Ok().json(json!({
                "message": "Successfully deleted"
            })))
        }

        Err(e) => {
            error!(error = %e, employee_id, "Failed to delete employee");

            Ok(HttpResponse::InternalServerError().json(json!({
                "message": "Internal Server Error"
            })))
        }
    }
}

/// Get Employee by ID
#[utoipa::path(
    get,
    path = "/api/v1/employees/{employee_id}",
    params(
        ("employee_id", Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Employee found", body = EmployeeResponse),
        (status = 404, description = "Employee not found", body = Object, example = json!({
            "message": "Employee not found"
        })),
        (status = 500, description = "Internal server error")
    ),
    tag = "Employee",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_employee(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> actix_web::Result<impl Responder> {
    let employee_id: u64 = path.into_inner();

    let employee = sqlx::query_as!(
        EmployeeResponse,
        r#"
        SELECT
            id,
            employee_code,
            first_name,
            last_name,
            email,
            department_id,
            job_title_id,
            hire_date,
            status
        FROM employees
        WHERE id = ?
        "#,
        employee_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, employee_id, "Failed to fetch employee");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    match employee {
        Some(emp) => Ok(HttpResponse::Ok().json(emp)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "Employee not found"
        }))),
    }
}
