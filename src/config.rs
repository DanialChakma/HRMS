use std::env;
use dotenvy::dotenv;
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_addr: String,
    pub access_token_ttl: usize,
    pub refresh_token_ttl: usize,

    // Rate limiting
    pub rate_login_per_min: u32,
    pub rate_register_per_min: u32,
    pub rate_refresh_per_min: u32,
    pub rate_protected_per_min: u32,

    pub api_prefix: String, // <-- new
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            server_addr: env::var("SERVER_ADDR").expect("SERVER_ADDR must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            access_token_ttl: env::var("ACCESS_TOKEN_TTL")
                .unwrap_or_else(|_| "900".to_string()) // default 15 min
                .parse()
                .unwrap(),
            refresh_token_ttl: env::var("REFRESH_TOKEN_TTL")
                .unwrap_or_else(|_| "604800".to_string()) // default 7 days
                .parse()
                .unwrap(),

            rate_login_per_min: env::var("RATE_LOGIN_PER_MIN")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap(),
            rate_register_per_min: env::var("RATE_REGISTER_PER_MIN")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap(),
            rate_refresh_per_min: env::var("RATE_REFRESH_PER_MIN")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap(),
            rate_protected_per_min: env::var("RATE_PROTECTED_PER_MIN")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap(),

            api_prefix: env::var("API_PREFIX").unwrap_or_else(|_| "/api".to_string()),
        }
    }
}
