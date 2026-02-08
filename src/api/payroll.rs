use actix_web::{HttpResponse, Responder, web};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use utoipa::{IntoParams, ToSchema};

use crate::auth::auth::AuthUser;

#[derive(Deserialize, ToSchema)]
pub struct CreatePayroll {
    #[schema(example = 1001)]
    pub employee_id: u64,

    #[schema(example = "2026-01-01", value_type = String, format = "date")]
    pub month: NaiveDate,

    #[schema(example = 50000.0)]
    pub base_salary: f64,

    #[schema(example = 5000.0)]
    pub bonus: f64,

    #[schema(example = 2000.0)]
    pub deductions: f64,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatePayroll {
    #[schema(example = 52000.0)]
    pub base_salary: Option<f64>,

    #[schema(example = 6000.0)]
    pub bonus: Option<f64>,

    #[schema(example = 2500.0)]
    pub deductions: Option<f64>,
}

#[derive(Serialize, ToSchema)]
pub struct PayrollResponse {
    pub id: u64,
    pub employee_id: u64,

    #[schema(value_type = String, format = "date")]
    pub month: NaiveDate,

    pub base_salary: f64,
    pub bonus: f64,
    pub deductions: f64,
    pub net_salary: f64,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct PayrollQuery {
    #[schema(example = 1)]
    pub page: Option<u32>,

    #[schema(example = 10)]
    pub per_page: Option<u32>,

    #[schema(example = 1001)]
    pub employee_id: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedPayrollResponse {
    pub data: Vec<PayrollResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

#[utoipa::path(
    post,
    path = "/api/v1/payroll",
    request_body = CreatePayroll,
    responses(
        (status = 201, description = "Payroll created"),
        (status = 401),
        (status = 403)
    ),
    security(("bearer_auth" = [])),
    tag = "Payroll"
)]
pub async fn create_payroll(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    payload: web::Json<CreatePayroll>,
) -> actix_web::Result<impl Responder> {
    auth.require_admin()?;

    let net_salary = payload.base_salary + payload.bonus - payload.deductions;

    sqlx::query!(
        r#"
        INSERT INTO payroll
        (employee_id, month, base_salary, bonus, deductions, net_salary)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        payload.employee_id,
        payload.month,
        payload.base_salary,
        payload.bonus,
        payload.deductions,
        net_salary,
    )
    .execute(pool.get_ref())
    .await;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Payroll created successfully"
    })))
}

#[utoipa::path(
    put,
    path = "/api/v1/payroll/{payroll_id}",
    request_body = UpdatePayroll,
    params(
        ("payroll_id", description = "Payroll ID")
    ),
    responses(
        (status = 200, description = "Payroll updated"),
        (status = 404, description = "Payroll not found")
    ),
    security(("bearer_auth" = [])),
    tag = "Payroll"
)]

pub async fn update_payroll(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    body: web::Json<UpdatePayroll>,
) -> actix_web::Result<impl Responder> {
    auth.require_admin()?;

    let payroll_id = path.into_inner();

    let current = sqlx::query!(
        r#"
    SELECT base_salary, bonus, deductions
    FROM payroll
    WHERE id = ?
    "#,
        payroll_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, payroll_id, "Failed to fetch payroll");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    let current = match current {
        Some(c) => c,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "message": "Payroll record not found"
            })));
        }
    };

    let base_salary = body.base_salary.unwrap_or(current.base_salary);
    let bonus = body.bonus.unwrap_or(current.bonus);
    let deductions = body.deductions.unwrap_or(current.deductions);
    let net_salary = base_salary + bonus - deductions;

    sqlx::query!(
        r#"
        UPDATE payroll
        SET base_salary = ?, bonus = ?, deductions = ?, net_salary = ?
        WHERE id = ?
        "#,
        base_salary,
        bonus,
        deductions,
        net_salary,
        payroll_id
    )
    .execute(pool.get_ref())
    .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Payroll updated successfully"
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/payroll/{payroll_id}",
    params(
        ("payroll_id", description = "Payroll ID")
    ),
    responses(
        (status = 200, body = PayrollResponse),
        (status = 404)
    ),
    security(("bearer_auth" = [])),
    tag = "Payroll"
)]
pub async fn get_payroll(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> actix_web::Result<impl Responder> {
    auth.require_admin()?;

    let payroll_id = path.into_inner();

    let payroll = sqlx::query_as!(
        PayrollResponse,
        r#"
        SELECT id, employee_id, month, base_salary, bonus, deductions, net_salary
        FROM payroll
        WHERE id = ?
        "#,
        payroll_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, payroll_id, "Failed to fetch payroll");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    match payroll {
        Some(p) => Ok(HttpResponse::Ok().json(p)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "Payroll not found"
        }))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/payroll",
    params(PayrollQuery),
    responses(
        (status = 200, body = PaginatedPayrollResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Payroll"
)]

pub async fn list_payrolls(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    query: web::Query<PayrollQuery>,
) -> actix_web::Result<impl Responder> {
    auth.require_admin()?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM payroll"#)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to count payrolls");
            actix_web::error::ErrorInternalServerError("Internal Server Error")
        })?;

    let data = sqlx::query_as!(
        PayrollResponse,
        r#"
        SELECT id, employee_id, month, base_salary, bonus, deductions, net_salary
        FROM payroll
        ORDER BY month DESC
        LIMIT ? OFFSET ?
        "#,
        per_page as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to fetch payroll list");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    Ok(HttpResponse::Ok().json(PaginatedPayrollResponse {
        data,
        page,
        per_page,
        total,
    }))
}

