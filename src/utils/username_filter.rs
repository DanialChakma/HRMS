use anyhow::{anyhow, Result};
use autoscale_cuckoo_filter::CuckooFilter;
use futures::StreamExt;
use once_cell::sync::Lazy;
use sqlx::MySqlPool;
use std::sync::RwLock;

/// Expected capacity and false-positive rate.
/// Tune these based on real user counts.
const FILTER_CAPACITY: usize = 100_000;
const FALSE_POSITIVE_RATE: f64 = 0.001;

static USERNAME_FILTER: Lazy<RwLock<CuckooFilter<String>>> = Lazy::new(|| {
    RwLock::new(CuckooFilter::new(
        FILTER_CAPACITY,
        FALSE_POSITIVE_RATE,
    ))
});

#[inline]
fn normalize(username: &str) -> String {
    username.to_lowercase()
}

/// Check if a username might exist (false positives possible)
pub fn might_exist(username: &str) -> bool {
    let username = normalize(username);
    USERNAME_FILTER
        .read()
        .expect("username filter poisoned")
        .contains(&username)
}

/// Insert a single username into the filter
pub fn insert(username: &str) {
    let username = normalize(username);
    USERNAME_FILTER
        .write()
        .expect("username filter poisoned")
        .add(&username);
}

/// Remove a username from the filter
pub fn remove(username: &str) {
    let username = normalize(username);
    USERNAME_FILTER
        .write()
        .expect("username filter poisoned")
        .remove(&username);
}

/// Warm up the username filter using streaming + batching
pub async fn warmup_username_filter(
    pool: &MySqlPool,
    batch_size: usize,
) -> Result<()> {
    let mut stream =
        sqlx::query_as::<_, (String,)>("SELECT username FROM users").fetch(pool);

    let mut batch = Vec::with_capacity(batch_size);
    let mut total = 0usize;

    while let Some(row) = stream.next().await {
        let (username,) =
            row.map_err(|e| anyhow!("DB row fetch failed: {}", e))?;

        batch.push(normalize(&username));
        total += 1;

        if batch.len() == batch_size {
            insert_batch(&batch);
            batch.clear();
        }
    }

    if !batch.is_empty() {
        insert_batch(&batch);
    }

    log::info!("Username filter warmup complete: {} users", total);
    Ok(())
}

/// Insert a batch of normalized usernames
fn insert_batch(usernames: &[String]) {
    let mut filter = USERNAME_FILTER
        .write()
        .expect("username filter poisoned");

    for username in usernames {
        filter.add(username);
    }
}
