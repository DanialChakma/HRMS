use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

//////////////////////////////
// Request DTOs
//////////////////////////////

#[derive(Deserialize, ToSchema)]
pub struct UserReq {
    #[schema(example = "john_doe")]
    pub username: String,

    #[schema(example = "supersecret123")]
    pub password: String,

    #[schema(example = 1)]
    pub role_id: u8,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginReqDto {
    #[schema(example = "john_doe")]
    pub username: String,

    #[schema(example = "supersecret123")]
    pub password: String,
}

//////////////////////////////
// Database models
//////////////////////////////

#[derive(FromRow, ToSchema)]
pub struct UserSql {
    #[schema(example = 1)]
    pub id: u64, // matches BIGINT UNSIGNED in DB

    #[schema(example = "john_doe")]
    pub username: String,

    #[schema(example = "hashedpassword123")]
    pub password: String,

    #[schema(example = 1)]
    pub role_id: u8,

    #[schema(example = 100, nullable)]
    pub employee_id: Option<u64>,
}

//////////////////////////////
// JWT Claims
//////////////////////////////

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    #[schema(example = 1)]
    pub user_id: u64,

    #[schema(example = "john_doe")]
    pub sub: String,

    #[schema(example = 1)]
    pub role: u8, // role id

    #[schema(example = 1672531199)]
    pub exp: usize,

    #[schema(example = "some-unique-jti")]
    pub jti: String,

    #[schema(example = "Access")]
    pub token_type: TokenType,

    #[schema(example = 100, nullable)]
    /// Present only if this user is linked to an employee record
    pub employee_id: Option<u64>,
}

//////////////////////////////
// TokenType Enum
//////////////////////////////

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}
