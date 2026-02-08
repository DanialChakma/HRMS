use crate::auth::auth::AuthUser;
use actix_web::{HttpResponse, Responder, web};
use sqlx::MySqlPool;

/// Check-in endpoint
#[utoipa::path(
    post,
    path = "/api/v1/attendance/check-in",
    responses(
        (status = 200, description = "Checked in successfully", body = Object, example = json!({
            "message": "Checked in successfully"
        })),
        (status = 400, description = "Already checked in today", body = Object, example = json!({
            "message": "Already checked in today"
        })),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Attendance"
)]
pub async fn check_in(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
) -> actix_web::Result<impl Responder> {
    let employee_id: u64 = auth
        .employee_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("No employee profile"))?;

    let result = sqlx::query!(
        r#"
        INSERT INTO attendance (employee_id, date, check_in)
        VALUES (?, CURDATE(), CURTIME())
        "#,
        employee_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Checked in successfully"
        }))),

        Err(e) => {
            // Duplicate check-in for same day
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23000") {
                    return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Already checked in today"
                    })));
                }
            }

            tracing::error!(error = %e, employee_id, "Check-in failed");
            Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ))
        }
    }
}

/// Check-out endpoint
#[utoipa::path(
    post,
    path = "/api/v1/attendance/check-out",
    responses(
        (status = 200, description = "Checked out successfully", body = Object, example = json!({
            "message": "Checked out successfully"
        })),
        (status = 400, description = "No active check-in found for today", body = Object, example = json!({
            "message": "No active check-in found for today"
        })),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Attendance"
)]
pub async fn check_out(
    auth: AuthUser,
    pool: web::Data<MySqlPool>,
) -> actix_web::Result<impl Responder> {
    let employee_id: u64 = auth
        .employee_id
        .ok_or_else(|| actix_web::error::ErrorForbidden("No employee profile"))?;

    let result = sqlx::query!(
        r#"
        UPDATE attendance
        SET check_out = CURTIME()
        WHERE employee_id = ?
        AND date = CURDATE()
        AND check_out IS NULL
        "#,
        employee_id
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, employee_id, "Check-out failed");
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "message": "No active check-in found for today"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Checked out successfully"
    })))
}
