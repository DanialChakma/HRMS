use crate::api::employee::{
    CreateEmployee, EmployeeListResponse, EmployeeResponse, UpdateEmployee,
};
use crate::api::leave_request::LeaveListResponse;
use crate::api::leave_request::LeaveResponse;
use crate::api::leave_request::{CreateLeave, LeaveFilter, LeaveType};
use crate::api::payroll::{
    CreatePayroll, PaginatedPayrollResponse, PayrollQuery, PayrollResponse, UpdatePayroll,
};
use crate::auth::handlers::LoginResponse;
use crate::model::employee::Employee;
use crate::models::{LoginReqDto, TokenType, UserReq};
use utoipa::Modify;

use utoipa::openapi::Components;
use utoipa::{OpenApi, openapi};
use utoipa::openapi::OpenApi as OpenApiStruct; // << important: this is the struct
// use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
#[derive(OpenApi)]
#[openapi(
    info(
        title = "HRM System API",
        version = "1.0.0",
        description = r#"
## Human Resource Management (HRM) System

This API powers a **Human Resource Management (HRM)** system designed to manage core HR operations within an organization.

### 🔹 Key Features
- **Employee Management**
  - Create, update, list, and view employee profiles
- **Leave Management**
  - Apply for leave, approve/reject requests, and view leave history
- **Attendance Management**
  - Daily check-in and check-out tracking
- **Payroll Management**
  - Generate payrolls, update salaries, and view payroll records

### 🔐 Security
Most endpoints are protected using **JWT Bearer authentication**.
Only authorized roles such as **Admin** or **HR** can access sensitive operations.

### 📦 Response Format
- JSON-based RESTful responses
- Pagination supported for list endpoints

### 🚀 Usage
Use this API to build:
- HR dashboards
- Employee self-service portals
- Payroll & attendance systems

---
Built with **Rust**, **Actix Web**, **SQLx**, and **Utoipa**.
"#,
    ),

    servers(
        (url = "http://localhost:3000", description = "Local development"),
        (url = "https://dev.hrm.co.uk", description = "Development"),
        (url = "https://preprod.hrm.co.uk", description = "Pre-production"),
        (url = "https://hrm.co.uk", description = "Production")
    ),

    paths(
        crate::api::leave_request::leave_list,
        crate::api::leave_request::get_leave,
        crate::api::leave_request::create_leave,
        crate::api::leave_request::approve_leave,
        crate::api::leave_request::reject_leave,

        crate::api::attendance::check_in,
        crate::api::attendance::check_out,

        crate::api::employee::create_employee,
        crate::api::employee::get_employee,
        crate::api::employee::list_employees,
        crate::api::employee::update_employee,

        crate::api::payroll::create_payroll,
        crate::api::payroll::update_payroll,
        crate::api::payroll::get_payroll,
        crate::api::payroll::list_payrolls,

        crate::auth::handlers::register,
        crate::auth::handlers::login,
        crate::auth::handlers::refresh_token,
        crate::auth::handlers::logout,
    ),
    components(
        schemas(
            LeaveFilter,
            CreateLeave,
            LeaveType,
            LeaveResponse,
            LeaveListResponse,
            CreateEmployee,
            UpdateEmployee,
            EmployeeResponse,
            Employee,
            EmployeeListResponse,
            PaginatedPayrollResponse,
            PayrollResponse,
            CreatePayroll,
            UpdatePayroll,
            PayrollQuery,

            UserReq,
            LoginReqDto,
            LoginResponse,
            TokenType
        ),
        // security_schemes(
        //     bearer_auth: SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer))
        // )
    ),

     modifiers(&AddSecurityScheme), // << injects the security scheme
    tags(
        (name = "Leave", description = "Leave management APIs"),
        (name = "Attendance", description = "Attendance management APIs"),
        (name = "Employee", description = "Employee management APIs"),
        (name = "Payroll", description = "Payroll management APIs"),
        (name = "Security", description = "Security: User authentication and authorization"),
    )
)]
pub struct ApiDoc;

pub struct AddSecurityScheme;

impl Modify for AddSecurityScheme {
    fn modify(&self, openapi: &mut OpenApiStruct) {
        // ensure components exist
        let components = openapi.components.get_or_insert_with(Components::default);

        // add bearer_auth scheme
        components
            .security_schemes
            .insert("bearer_auth".to_string(), SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
    }
}