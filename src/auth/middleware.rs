use crate::auth::auth::AuthUser;
use crate::auth::jwt::verify_token;
use crate::config::Config;
use crate::model::role::Role;
use actix_web::error::ErrorUnauthorized;
use actix_web::middleware::Next;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    web::Data,
};
use serde_json::json;
pub async fn auth_middleware(
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let config = req
        .app_data::<Data<Config>>()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("App config missing"))?;

    let header_value = match req.headers().get("Authorization") {
        Some(h) => h.to_str().map_err(|_| {
            actix_web::error::ErrorUnauthorized(
                json!({"error": "Invalid Authorization header encoding"}),
            )
        })?,
        None => {
            let resp =
                HttpResponse::Unauthorized().json(json!({"error": "Missing Authorization header"}));
            return Ok(req.into_response(resp.map_into_boxed_body()));
        }
    };

    let token = match header_value.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            let resp = HttpResponse::Unauthorized()
                .json(json!({"error": "Authorization header must start with Bearer"}));
            return Ok(req.into_response(resp.map_into_boxed_body()));
        }
    };

    let claims = match verify_token(token, &config.jwt_secret) {
        Ok(c) => c,
        Err(e) => {
            let resp = HttpResponse::Unauthorized()
                .json(json!({"error": "Invalid or expired token", "details": e}));
            return Ok(req.into_response(resp.map_into_boxed_body()));
        }
    };

    let role = match Role::from_id(claims.role) {
        Some(role) => role,
        None => {
            let resp =
                HttpResponse::Unauthorized().json(json!({"error": "Invalid role"}));
            return Ok(req.into_response(resp.map_into_boxed_body()));
        }
    };

    let auth_user = AuthUser {
        user_id: claims.user_id,
        username: claims.sub,
        role,
        employee_id: claims.employee_id,
    };

    req.extensions_mut().insert(auth_user);

    next.call(req).await
}

