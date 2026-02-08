use crate::api::employee::{
    CreateEmployee, EmployeeListResponse, EmployeeResponse, UpdateEmployee,
};
use crate::api::leave_request::LeaveFilter;
use crate::api::leave_request::LeaveListResponse;
use crate::api::leave_request::LeaveResponse;
use crate::api::payroll::{
    CreatePayroll, PaginatedPayrollResponse, PayrollQuery, PayrollResponse, UpdatePayroll,
};
use crate::model::employee::Employee;
use utoipa::Modify;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{OpenApi, openapi};
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
        crate::api::payroll::list_payrolls
    ),
    components(
        schemas(
            LeaveFilter,
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
            PayrollQuery
        )
    ),
    tags(
        (name = "Leave", description = "Leave management APIs"),
        (name = "Attendance", description = "Attendance management APIs"),
        (name = "Employee", description = "Employee management APIs"),
        (name = "Payroll", description = "Payroll management APIs"),
    )
)]
pub struct ApiDoc;
