use crate::{
    auth::{
        jwt::{generate_access_token, generate_refresh_token, verify_token},
        password::{hash_password, verify_password},
    },
    config::Config,
    models::{LoginReqDto, TokenType, UserReq, UserSql},
};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;
use tracing::{debug, error, info, instrument};
// use scalable_cuckoo_filter::ScalableCuckooFilter;
use crate::utils::username_cache;
use crate::utils::username_filter;
// auth end points

/// Inserts a new user into the database and updates the Cuckoo filter
async fn insert_user(
    username: &str,
    password: &str,
    role: &u8,
    pool: &MySqlPool,
) -> Result<(), HttpResponse> {
    let hashed = hash_password(password);

    let result = sqlx::query!(
        r#"INSERT INTO users (username, password) VALUES (?, ?)"#,
        username,
        hashed
    )
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            // if insert succcess, insert/populate filter using username. and keep cache populated.
            username_filter::insert(username);
            username_cache::mark_taken(username);
            Ok(())
        }
        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code() == Some("23000".into()) {
                    return Err(HttpResponse::Conflict().json(json!({
                        "error": "Username already exists"
                    })));
                }
            }

            Err(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to register user"
            })))
        }
    }
}

/// true  => username AVAILABLE
/// false => username TAKEN
pub async fn is_username_available(username: &str, pool: &MySqlPool) -> bool {
    let username = username.to_lowercase();

    // Fast in-memory check with Cuckoo filter
    // 1️⃣ Cuckoo filter — fast negative
    // if filter says not exist then it is saying true, else it may exist or not.
    if !username_filter::might_exist(&username) {
        return true;
    }

    // 2️⃣ Moka cache — fast positive
    if username_cache::is_taken(&username).await {
        // if username is taken then is_available equal false
        return false;
    }

    // 3️⃣ Database fallback
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? LIMIT 1)",
    )
    .bind(&username)
    .fetch_one(pool)
    .await
    .unwrap_or(true); // fail-safe

    if exists {
        // username_filter::insert(&username);
        // username_cache::mark_taken(&username).await;
        return false;
    }

    true
}

// #[post("/register")]

/// User registration handler
pub async fn register(user: web::Json<UserReq>, pool: web::Data<MySqlPool>) -> impl Responder {
    let username = user.username.trim();
    let password = &user.password;
    let role = &user.role_id;

    if username.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Username and password must not be empty"
        }));
    }

    if !is_username_available(&user.username, pool.get_ref()).await {
        return HttpResponse::Conflict().json(json!({
            "error": "Username already taken"
        }));
    }

    // Safe to insert after DB check
    match insert_user(username, password, role, pool.get_ref()).await {
        Ok(_) => HttpResponse::Created().json(json!({
            "message": "User registered successfully"
        })),
        Err(err_resp) => err_resp,
    }
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
}

// #[post("/login")]
#[instrument(
    name = "auth_login",
    skip(pool, config, user),
    fields(username = %user.username)
)]
pub async fn login(
    user: web::Json<LoginReqDto>,
    pool: web::Data<MySqlPool>,
    config: web::Data<Config>,
) -> impl Responder {
    info!("Login request received");

    // 1️⃣ Basic validation
    if user.username.trim().is_empty() || user.password.is_empty() {
        info!("Validation failed: empty username or password");
        return HttpResponse::BadRequest().body("Username or password required");
    }

    debug!("Fetching user from database");

    // 2️⃣ Fetch user
    let db_user = match sqlx::query_as::<_, UserSql>(
        r#"
        SELECT id, username, password, role_id, employee_id
        FROM users
        WHERE username = ?
        "#,
    )
    .bind(&user.username)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(user)) => {
            debug!(user_id = user.id, "User found");
            user
        }
        Ok(None) => {
            info!("Invalid credentials: user not found");
            return HttpResponse::Unauthorized().body("Invalid credentials");
        }
        Err(e) => {
            error!(error = %e, "Database error while fetching user");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 3️⃣ Verify password
    debug!("Verifying password");

    if let Err(e) = verify_password(&user.password, &db_user.password) {
        info!(error = %e, "Invalid credentials: password mismatch");
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    debug!("Password verified");

    // 4️⃣ Generate access token
    debug!("Generating access token");

    let access_token = generate_access_token(
        db_user.id,
        db_user.username.clone(),
        db_user.role_id,
        db_user.employee_id,
        &config.jwt_secret,
        config.access_token_ttl,
    );

    // 5️⃣ Generate refresh token
    debug!("Generating refresh token");

    let (refresh_token, refresh_claims) = generate_refresh_token(
        db_user.id,
        db_user.username.clone(),
        db_user.role_id,
        db_user.employee_id,
        &config.jwt_secret,
        config.refresh_token_ttl,
    );

    // 6️⃣ Store refresh token
    debug!(
        user_id = db_user.id,
        jti = %refresh_claims.jti,
        "Storing refresh token"
    );

    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO refresh_tokens (user_id, jti, expires_at)
        VALUES (?, ?, FROM_UNIXTIME(?))
        "#,
    )
    .bind(db_user.id)
    .bind(&refresh_claims.jti)
    .bind(refresh_claims.exp as i64)
    .execute(pool.get_ref())
    .await
    {
        error!(error = %e, "Failed to store refresh token");
        return HttpResponse::InternalServerError().finish();
    }

    // 7️⃣ Update last_login_at (non-fatal)
    debug!("Updating last_login_at");

    if let Err(e) = sqlx::query!(
        "UPDATE users SET last_login_at = NOW() WHERE username = ?",
        user.username
    )
    .execute(pool.get_ref())
    .await
    {
        error!(error = %e, "Failed to update last_login_at");
        // intentionally not failing login
    }

    info!("Login successful");

    HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
    })
}

#[get("/protected")]
pub async fn protected(req: HttpRequest) -> impl Responder {
    match req.extensions().get::<String>() {
        Some(user) => HttpResponse::Ok().body(user.clone()),
        None => HttpResponse::Unauthorized().body("No user"),
    }
}

// #[post("/refresh")]
pub async fn refresh_token(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    config: web::Data<Config>,
) -> impl Responder {
    let header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::Unauthorized().body("No token"),
    };

    let token = match header.strip_prefix("Bearer ") {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let claims = match verify_token(token, &config.jwt_secret) {
        Ok(c) => c,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    if claims.token_type != TokenType::Refresh {
        return HttpResponse::Unauthorized().finish();
    }

    // 🔍 find refresh token in DB
    let record = sqlx::query!(
        r#"
        SELECT id, user_id, revoked
        FROM refresh_tokens
        WHERE jti = ?
        "#,
        claims.jti
    )
    .fetch_optional(pool.get_ref())
    .await
    .unwrap();

    let record = match record {
        Some(r) if r.revoked == 0 => r, // 0=not revoked
        _ => return HttpResponse::Unauthorized().finish(),
    };

    // 🔥 revoke old refresh token
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked = TRUE WHERE id = ?",
        record.id
    )
    .execute(pool.get_ref())
    .await
    .unwrap();

    // 🔄 issue new refresh token
    let (new_refresh_token, new_claims) = generate_refresh_token(
        claims.user_id.clone(),
        claims.sub.clone(),
        claims.role.clone(),
        claims.employee_id,
        &config.jwt_secret,
        config.refresh_token_ttl,
    );

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, jti, expires_at)
        VALUES (?, ?, FROM_UNIXTIME(?))
        "#,
        record.user_id,
        new_claims.jti,
        new_claims.exp as i64
    )
    .execute(pool.get_ref())
    .await
    .unwrap();

    // 🎫 new access token
    let access_token = generate_access_token(
        claims.user_id.clone(),
        claims.sub.clone(),
        claims.role.clone(),
        claims.employee_id,
        &config.jwt_secret,
        config.access_token_ttl,
    );

    HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": new_refresh_token
    }))
}

// #[post("/logout")]
pub async fn logout(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    config: web::Data<Config>,
) -> impl Responder {
    // 1️⃣ extract Authorization header
    let header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return HttpResponse::NoContent().finish(),
    };

    let token = match header.strip_prefix("Bearer ") {
        Some(t) => t,
        None => return HttpResponse::NoContent().finish(),
    };

    // 2️⃣ verify JWT
    let claims = match verify_token(token, &config.jwt_secret) {
        Ok(c) => c,
        Err(_) => return HttpResponse::NoContent().finish(),
    };

    // 3️⃣ only refresh tokens can logout
    if claims.token_type != TokenType::Refresh {
        return HttpResponse::NoContent().finish();
    }

    // 4️⃣ revoke refresh token (idempotent)
    let _ = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked = 1
        WHERE jti = ?
        "#,
        claims.jti
    )
    .execute(pool.get_ref())
    .await;

    // 5️⃣ success (even if token didn't exist)
    HttpResponse::NoContent().finish()
}
