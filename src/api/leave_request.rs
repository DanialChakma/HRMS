use crate::auth::auth::AuthUser;
use actix_web::{HttpResponse, Responder, web};
// use chrono::NaiveDateTime;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, prelude::FromRow};
// use utoipa::path as utoipa_path;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LeaveType {
    Annual,
    Sick,
    Unpaid,
}

impl LeaveType {
    fn as_str(&self) -> &str {
        match self {
            LeaveType::Annual => "annual",
            LeaveType::Sick => "sick",
            LeaveType::Unpaid => "unpaid",
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateLeave {
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    pub start_date: chrono::NaiveDate,
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    pub end_date: chrono::NaiveDate,
    #[schema(
        example = "sick"
    )]
    pub leave_type: LeaveType,  // enum ensures Swagger dropdown
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({
    "data": [
        {
            "id": 1,
            "employee_id": 1000,
            "start_date": "2026-01-01",
            "end_date": "2026-01-03",
            "leave_type": "sick",
            "status": "pending",
            "created_at": "2026-01-01T00:00:00Z"
        }
    ],
    "page": 1,
    "per_page": 10,
    "total": 1
}))]
pub struct LeaveListResponse {
    #[schema(example = json!([{
    "id": 1,
    "employee_id": 1000,
    "start_date": "2026-01-01",
    "end_date": "2026-01-03",
    "leave_type": "sick",
    "status": "pending",
    "created_at": "2026-01-01T00:00:00Z"
}]))]
    pub data: Vec<LeaveResponse>,
    #[schema(example = 1)]
    pub page: u32,
    #[schema(example = 10)]
    pub per_page: u32,
    #[schema(example = 1)]
    pub total: i64,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct LeaveFilter {
    #[schema(example = 123)]
    /// Filter by employee ID
    pub employee_id: Option<u64>,
    #[schema(example = "Pending")]
    /// Filter by leave status
    pub status: Option<String>,
    #[schema(example = 1)]
    /// Pagination page number (start with 1)
    pub page: Option<u64>, // 1-based
    #[schema(example = 3)]
    /// Pagination per page number
    pub per_page: Option<u64>, // items per page
}

// Helper enum for typed SQLx binding
enum FilterValue<'a> {
    U64(u64),
    Str(&'a str),
}

#[derive(Serialize, Deserialize, FromRow, ToSchema)]
pub struct LeaveResponse {
    #[schema(example = 1)]
    /// leave application id
    pub id: u64,
    /// employee id for whom the leave is applied
    #[schema(example = 1000)]
    pub employee_id: u64,
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    /// leave start date
    pub start_date: chrono::NaiveDate,
    // leave end date
    #[schema(example = "2026-01-01", format = "date", value_type = String)]
    pub end_date: chrono::NaiveDate,
    #[schema(example = "sick", value_type = String)]
    // leave type
    pub leave_type: String,
    #[schema(example = "pending", value_type = String )]
    // leave status
    pub status: Option<String>,
    // leave creation date time
    #[schema(example = "2026-01-01T00:00:00Z", format = "date-time", value_type = String)]
    pub created_at: Option<DateTime<Utc>>,
}

fn validate_leave_type(value: &LeaveType) -> bool {
    matches!(value, LeaveType::Annual | LeaveType::Sick | LeaveType::Unpaid)
}

/* =========================
Create leave request
========================= */
/// Swagger doc for create_leave endpoint
#[utoipa::path(
    post,
    path = "/api/v1/leave",
    request_body(
        content = CreateLeave,
        description = "Leave request payload",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Leave request submitted successfully",
         body = Object,
         example = json!({
            "message": "Leave request submitted",
            "status": "pending"
         })
        ),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Leave"
)]
pub async fn create_leave(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    payload: web::Json<CreateLeave>,
) -> actix_web::Result<impl Responder> {
    let employee_id: u64 = auth
        .employee_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("No employee profile"))?;

    // 1️⃣ validate dates
    if payload.start_date > payload.end_date {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "start_date cannot be after end_date"
        })));
    }

    // 2️⃣ validate leave type
    if !validate_leave_type(&payload.leave_type) {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Invalid leave type. Allowed: annual, sick, unpaid"
        })));
    }

    // 3️⃣ insert request
    sqlx::query!(
        r#"
        INSERT INTO leave_requests
            (employee_id, start_date, end_date, leave_type)
        VALUES (?, ?, ?, ?)
        "#,
        employee_id,
        payload.start_date,
        payload.end_date,
        payload.leave_type.as_str(),
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, employee_id, "Failed to create leave request");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Leave request submitted",
        "status": "pending"
    })))
}

/* =========================
Approve leave (HR/Admin)
========================= */
/// Swagger doc for approve_leave endpoint
#[utoipa::path(
    put,
    path = "/api/v1/leave/{leave_id}/approve",
    params(
        ("leave_id" = u64, Path, description = "ID of the leave request to approve")
    ),
    responses(
        (status = 200, description = "Leave approved successfully", body = Object, example = json!({
            "message": "Leave approved"
        })),
        (status = 400, description = "Leave request not found or already processed", body = Object, example = json!({
            "message": "Leave request not found or already processed"
        })),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Leave"
)]
pub async fn approve_leave(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> actix_web::Result<impl Responder> {
    auth.require_hr_or_admin()?;

    let leave_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE leave_requests
        SET status = 'approved'
        WHERE id = ?
        AND status = 'pending'
        "#,
        leave_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, leave_id, "Approve leave failed");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Leave request not found or already processed"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Leave approved"
    })))
}

/* =========================
Reject leave (HR/Admin)
========================= */
/// Swagger doc for reject_leave endpoint
#[utoipa::path(
    put,
    path = "/api/v1/leave/{leave_id}/reject",
    params(
        ("leave_id" = u64, Path, description = "ID of the leave request to reject")
    ),
    responses(
        (status = 200, description = "Leave rejected successfully", body = Object, example = json!({
            "message": "Leave rejected"
        })),
        (status = 400, description = "Leave request not found or already processed", body = Object, example = json!({
            "message": "Leave request not found or already processed"
        })),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Leave"
)]

pub async fn reject_leave(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> actix_web::Result<impl Responder> {
    auth.require_hr_or_admin()?;

    let leave_id = path.into_inner();

    let result = sqlx::query!(
        r#"
        UPDATE leave_requests
        SET status = 'rejected'
        WHERE id = ?
        AND status = 'pending'
        "#,
        leave_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, leave_id, "Reject leave failed");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Leave request not found or already processed"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Leave rejected"
    })))
}

/// for getting a leave application details endpoint
#[utoipa::path(
    get,
    path = "/api/v1/leave/{leave_id}",
    params(
        ("leave_id" = u64, Path, description = "ID of the leave request to fetch")
    ),
    responses(
        (status = 200, description = "Leave request found", body = LeaveResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Leave request not found", body = Object, example = json!({
            "message": "Leave request not found"
        }))
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Leave"
)]
pub async fn get_leave(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
) -> actix_web::Result<impl Responder> {
    auth.require_hr_or_admin()?;

    let leave_id = path.into_inner();

    let leave = sqlx::query_as!(
        LeaveResponse,
        r#"
        SELECT
            id,
            employee_id,
            start_date,
            end_date,
            leave_type AS `leave_type: String`,
            status,
            created_at
        FROM leave_requests
        WHERE id = ?
        "#,
        leave_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, leave_id, "Failed to fetch leave request");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    match leave {
        Some(data) => Ok(HttpResponse::Ok().json(data)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": "Leave request not found"
        }))),
    }
}

/// for getting leave applications endpoint
#[utoipa::path(
    get,
    path = "/api/v1/leave",
    params(LeaveFilter),
    responses(
        (status = 200, description = "Paginated leave list", body = LeaveListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Leave"
)]
pub async fn leave_list(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
    query: web::Query<LeaveFilter>,
) -> actix_web::Result<impl Responder> {
    auth.require_hr_or_admin()?;

    // -------------------------
    // Pagination
    // -------------------------
    let per_page = query.per_page.unwrap_or(10).min(100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    // -------------------------
    // WHERE clause
    // -------------------------
    let mut where_sql = String::from(" WHERE 1=1");
    let mut args: Vec<FilterValue> = Vec::new();

    if let Some(emp_id) = query.employee_id {
        where_sql.push_str(" AND employee_id = ?");
        args.push(FilterValue::U64(emp_id));
    }

    if let Some(status) = query.status.as_deref() {
        where_sql.push_str(" AND status = ?");
        args.push(FilterValue::Str(status));
    }

    // -------------------------
    // COUNT query
    // -------------------------
    let count_sql = format!("SELECT COUNT(*) FROM leave_requests{}", where_sql);

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for arg in &args {
        count_q = match arg {
            FilterValue::U64(v) => count_q.bind(*v),
            FilterValue::Str(s) => count_q.bind(*s),
        };
    }

    let total = count_q.fetch_one(pool.get_ref()).await.map_err(|e| {
        tracing::error!(error=%e, "Failed to count leave requests");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    // -------------------------
    // DATA query
    // -------------------------
    let data_sql = format!(
        r#"
        SELECT id, employee_id, start_date, end_date, leave_type, status, created_at
        FROM leave_requests
        {}
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
        where_sql
    );

    let mut data_q = sqlx::query_as::<_, LeaveResponse>(&data_sql);
    for arg in args {
        data_q = match arg {
            FilterValue::U64(v) => data_q.bind(v),
            FilterValue::Str(s) => data_q.bind(s),
        };
    }

    let leaves = data_q
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!(error=%e, "Failed to fetch leave list");
            actix_web::error::ErrorInternalServerError("Internal Server Error")
        })?;

    // -------------------------
    // Response
    // -------------------------
    let response = LeaveListResponse {
        data: leaves,
        page: page as u32,
        per_page: per_page as u32,
        total,
    };

    Ok(HttpResponse::Ok().json(response))
}
