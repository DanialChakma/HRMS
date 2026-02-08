use anyhow::Result;
use futures_util::StreamExt;
use moka::future::Cache;
use once_cell::sync::Lazy;
use sqlx::MySqlPool;
use std::time::Duration;

/// true  => username is TAKEN
/// false => username is AVAILABLE (usually we store only taken)
pub static USERNAME_CACHE: Lazy<Cache<String, bool>> = Lazy::new(|| {
    Cache::builder()
        .max_capacity(500_000) // tune based on memory
        .time_to_live(Duration::from_secs(86400)) // 24h TTL
        .build()
});

/// Mark a single username as taken
pub async fn mark_taken(username: &str) {
    USERNAME_CACHE
        .insert(username.to_lowercase(), true)
        .await;
}

/// Check if username is taken
pub async fn is_taken(username: &str) -> bool {
    USERNAME_CACHE
        .get(&username.to_lowercase())
        .await
        .unwrap_or(false)
}

/// Batch mark usernames as taken
async fn batch_mark(usernames: &[String]) {
    let futures: Vec<_> = usernames
        .iter()
        .map(|u| USERNAME_CACHE.insert(u.to_lowercase(), true))
        .collect();

    // Await all insertions concurrently
    futures::future::join_all(futures).await;
}

/// Load only RECENT usernames into in-memory cache (batched)
pub async fn warmup_username_cache(
    pool: &MySqlPool,
    days: u32,
    batch_size: usize,
) -> Result<()> {
    let mut stream = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT username
        FROM users
        WHERE last_login_at >= NOW() - INTERVAL ? DAY
        ORDER BY last_login_at DESC
        "#,
    )
    .bind(days)
    .fetch(pool);

    let mut batch = Vec::with_capacity(batch_size);
    let mut total_count = 0usize;

    while let Some(row) = stream.next().await {
        let (username,) = row?;
        batch.push(username);
        total_count += 1;

        if batch.len() >= batch_size {
            batch_mark(&batch).await;
            batch.clear();
        }
    }

    // Insert any remaining usernames
    if !batch.is_empty() {
        batch_mark(&batch).await;
    }

    log::info!(
        "Username cache warmup complete: {} recent users (last {} days)",
        total_count,
        days
    );

    Ok(())
}
