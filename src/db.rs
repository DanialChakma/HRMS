// use sqlx::PgPool;

// pub async fn init_db(database_url: &str) -> PgPool {
//     PgPool::connect(database_url)
//         .await
//         .expect("Failed to connect to database")
// }


use sqlx::MySqlPool;

pub async fn init_db(database_url: &str) -> MySqlPool {
    MySqlPool::connect(database_url)
        .await
        .expect("Failed to connect to database")
}
