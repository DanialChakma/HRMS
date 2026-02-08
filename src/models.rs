use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct UserReq {
    pub username: String,
    pub password: String,
    pub role_id: u8,
}

#[derive(Deserialize)]
pub struct LoginReqDto {
    pub username: String,
    pub password: String
}

#[derive(FromRow)]
pub struct UserSql {
    pub id: u64,        // 👈 matches BIGINT UNSIGNED,
    pub username: String,
    pub password: String,
    pub role_id: u8,
    pub employee_id: Option<u64>
    // pub employee_id: Option<u64>,
    // pub is_active: bool,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: u64,
    pub sub: String,
    pub role: u8,        // role id
    pub exp: usize,
    pub jti: String,

    pub token_type: TokenType,
    /// Present only if this user is linked to an employee record
    pub employee_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}
