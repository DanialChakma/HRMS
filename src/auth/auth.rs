use crate::{model::role::Role, models::Claims};
use actix_web::{FromRequest, HttpRequest, dev::Payload, error::ErrorUnauthorized, web::Data};
use futures::future::{Ready, ready};
use jsonwebtoken::decode;
use jsonwebtoken::{DecodingKey, Validation};
use crate::config::Config;
pub struct AuthUser {
    pub user_id: u64,
    pub username: String,
    pub role: Role,

    /// Present only if this user is linked to an employee record
    pub employee_id: Option<u64>,
}

impl FromRequest for AuthUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = match req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
        {
            Some(t) => t,
            None => return ready(Err(ErrorUnauthorized("Missing token"))),
        };

        let config = match req.app_data::<Data<Config>>() {
            Some(c) => c,
            None => {
                return ready(Err(
                    actix_web::error::ErrorInternalServerError("Config missing"),
                ))
            }
        };

        let data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(d) => d,
            Err(_) => return ready(Err(ErrorUnauthorized("Invalid token"))),
        };

        let role = match Role::from_id(data.claims.role) {
            Some(r) => r,
            None => return ready(Err(ErrorUnauthorized("Invalid role"))),
        };

        ready(Ok(AuthUser {
            user_id: data.claims.user_id,
            username: data.claims.sub,
            role,
            employee_id: data.claims.employee_id,
        }))
    }
}

impl AuthUser {
    pub fn require_admin(&self) -> actix_web::Result<()> {
        if self.role == Role::Admin {
            Ok(())
        } else {
            Err(actix_web::error::ErrorForbidden("Admin only"))
        }
    }

    pub fn require_hr_or_admin(&self) -> actix_web::Result<()> {
        if matches!(self.role, Role::Admin | Role::Hr) {
            Ok(())
        } else {
            Err(actix_web::error::ErrorForbidden("HR/Admin only"))
        }
    }

    /// Returns true if the user is an employee
    pub fn is_employee(&self) -> bool {
        self.role == Role::Employee
    }
}
