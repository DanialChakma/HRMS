use actix_web::middleware::NormalizePath;
use actix_web::web::Data;
use actix_web::{App, HttpServer, Responder, get};
use dotenvy::dotenv;

mod api;
mod auth;
mod config;
mod db;
mod model;
mod models;
mod routes;
mod utils;
mod docs;

use config::Config;
use db::init_db;

use crate::utils::username_cache;
use crate::utils::username_filter;
use tracing::{info, warn};
use tracing_appender::rolling;
// use tracing_subscriber::fmt::writer::BoxMakeWriter;
use utoipa_swagger_ui::SwaggerUi;
use crate::docs::ApiDoc;
use utoipa::OpenApi; // ← needed for ApiDoc::openapi()

#[get("/")]
async fn index() -> impl Responder {
    "Hello World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::from_env();

    // Rolling daily log
    let file_appender = rolling::daily("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_target(false) // removes module path
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .pretty()
        .init();

    info!("Server starting...");

    let pool = init_db(&config.database_url).await;

    let pool_for_filter_warmup = pool.clone();
    let pool_for_cache_warmup = pool.clone();
    // 👇 clone what you need BEFORE moving config
    // Clone values for the closure (avoid move issues)
    let server_addr = config.server_addr.clone();
    let config_data = config.clone();

    actix_web::rt::spawn(async move {
        if let Err(e) = username_filter::warmup_username_filter(&pool_for_filter_warmup, 100).await
        {
            eprintln!("Failed to warmup username filter: {:?}", e);
        }
    });

    actix_web::rt::spawn(async move {
        // Warm up last 30 days of recent users in batches of 500
        if let Err(e) = username_cache::warmup_username_cache(&pool_for_cache_warmup, 30, 250).await
        {
            eprintln!("Failed to warmup username cache: {:?}", e);
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(NormalizePath::trim())
            .service(
                // SwaggerUi::new("/swagger-ui/{_:.*}")
                SwaggerUi::new("/swagger-ui/{_:.*}") // ← important: wildcard {_:.*} to match JS/CSS files
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(config.clone()))
            .service(index)
            // Configure auth + protected routes with rate limiting
            .configure(|cfg| routes::configure(cfg, config_data.clone()))
    })
    .bind(server_addr)?
    .run()
    .await
}
