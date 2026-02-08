use crate::{
    api::{attendance, employee, leave_request, payroll},
    auth::{handlers, middleware::auth_middleware},
    config::Config,
};
use actix_governor::{
    Governor, GovernorConfigBuilder, PeerIpKeyExtractor, governor::middleware::NoOpMiddleware,
};
use actix_web::{middleware::from_fn, web};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig, config: Config) {
    // Helper to build per-route limiter
    fn build_limiter(requests_per_min: u32) -> Governor<PeerIpKeyExtractor, NoOpMiddleware> {
        let per_ms = if requests_per_min == 0 {
            1
        } else {
            60_000 / requests_per_min as u64
        };
        let cfg = GovernorConfigBuilder::default()
            .per_millisecond(per_ms)
            .burst_size(requests_per_min)
            .key_extractor(PeerIpKeyExtractor)
            .finish()
            .unwrap();
        Governor::new(&cfg)
    }

    let login_limiter = Arc::new(build_limiter(config.rate_login_per_min));
    let register_limiter = Arc::new(build_limiter(config.rate_register_per_min));
    let refresh_limiter = Arc::new(build_limiter(config.rate_refresh_per_min));
    let protected_limiter = Arc::new(build_limiter(config.rate_protected_per_min));

    // Public routes
    cfg.service(
        web::scope("/auth")
            .service(
                web::resource("/login")
                    .wrap(login_limiter.clone())
                    .route(web::post().to(handlers::login)),
            )
            .service(
                web::resource("/register")
                    .wrap(register_limiter.clone())
                    .route(web::post().to(handlers::register)),
            )
            .service(
                web::resource("/refresh")
                    .wrap(refresh_limiter.clone())
                    .route(web::post().to(handlers::refresh_token)),
            )
            .service(
                web::resource("/logout")
                    .wrap(login_limiter.clone())
                    .route(web::post().to(handlers::logout)),
            ),
    );

    // Protected routes
    cfg.service(
        web::scope(&config.api_prefix)
            .wrap(from_fn(auth_middleware))
             // authentication
            .wrap(protected_limiter) // rate limiting
            .service(handlers::protected)
            .service(
                web::scope("/employee")
                    // /employee
                    .service(
                        web::resource("")
                            .route(web::post().to(employee::create_employee))
                            .route(web::get().to(employee::list_employees)),
                    )
                    // /employee/{id}
                    .service(
                        web::resource("/{id}")
                            .route(web::put().to(employee::update_employee))
                            .route(web::get().to(employee::get_employee))
                            .route(web::delete().to(employee::delete_employee)),
                    ),
            )
            .service(
                web::scope("/leave")
                    // /leave
                    .service(
                        web::resource("")
                            .route(web::get().to(leave_request::leave_list))
                            .route(web::post().to(leave_request::create_leave)),
                    )
                    // /leave/{id}
                    .service(web::resource("/{id}").route(web::get().to(leave_request::get_leave)))
                    // /leave/{id}/approve
                    .service(
                        web::resource("/{id}/approve")
                            .route(web::put().to(leave_request::approve_leave)),
                    )
                    // /leave/{id}/reject
                    .service(
                        web::resource("/{id}/reject")
                            .route(web::put().to(leave_request::reject_leave)),
                    ),
            )
            .service(
                web::scope("/attendance")
                    // /attendance
                    .service(
                        web::resource("")
                            .route(web::put().to(attendance::check_out))
                            .route(web::post().to(attendance::check_in)),
                    )
            )
            .service(
                web::scope("/payroll")
                    // /payroll
                    .service(
                        web::resource("")
                            .route(web::post().to(payroll::create_payroll))
                            .route(web::put().to(payroll::update_payroll))
                            .route(web::get().to(payroll::list_payrolls))
                    )
                    //payroll/{id}
                    .service(web::resource("/{id}").route(web::get().to(payroll::get_payroll)))
            )
            ,
    );
}

// LOGIN
//  ├─ access_token (15 min)
//  └─ refresh_token (7 days)

// API REQUEST
//  └─ Authorization: Bearer access_token

// ACCESS EXPIRED
//  └─ POST /refresh with refresh_token
//       └─ returns new access_token
