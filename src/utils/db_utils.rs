use serde_json::Value;
use chrono::{NaiveDate, NaiveDateTime};
use actix_web::error::ErrorBadRequest;
use sqlx::MySqlPool;


/// ===============================
/// SQL bindable value enum
/// ===============================
#[derive(Debug)]
pub enum SqlValue {
    String(String),
    I64(i64),
    F64(f64),
    Bool(bool),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Null,
}


/// ===============================
/// SQL update container
/// ===============================
#[derive(Debug)]
pub struct SqlUpdate {
    pub sql: String,
    pub values: Vec<SqlValue>,
}


/// ===============================
/// Build dynamic UPDATE SQL
/// ===============================
pub fn build_update_sql(
    table: &str,
    payload: &Value,
    id_column: &str,
    id_value: i64,
) -> Result<SqlUpdate, actix_web::Error> {
    let obj = payload
        .as_object()
        .ok_or_else(|| ErrorBadRequest("Payload must be a JSON object"))?;

    if obj.is_empty() {
        return Err(ErrorBadRequest("No fields provided for update"));
    }

    // Build SET clause
    let set_clause = obj
        .keys()
        .map(|k| format!("{} = ?", k))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        table, set_clause, id_column
    );

    let mut values = Vec::with_capacity(obj.len() + 1);

    // Convert JSON values → SqlValue
    for value in obj.values() {
        match value {
            Value::String(s) => {
                if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    values.push(SqlValue::Date(d));
                } else if let Ok(dt) =
                    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                {
                    values.push(SqlValue::DateTime(dt));
                } else {
                    values.push(SqlValue::String(s.clone()));
                }
            }
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    values.push(SqlValue::I64(i));
                } else if let Some(f) = n.as_f64() {
                    values.push(SqlValue::F64(f));
                }
            }
            Value::Bool(b) => values.push(SqlValue::Bool(*b)),
            Value::Null => values.push(SqlValue::Null),
            _ => return Err(ErrorBadRequest("Unsupported JSON value type")),
        }
    }

    // WHERE id = ?
    values.push(SqlValue::I64(id_value));

    Ok(SqlUpdate { sql, values })
}


/// ===============================
/// Execute the update
/// ===============================
pub async fn execute_update(
    pool: &MySqlPool,
    update: SqlUpdate,
) -> Result<u64, sqlx::Error> {
    let mut query = sqlx::query(&update.sql);

    for value in update.values {
        query = match value {
            SqlValue::String(v) => query.bind(v),
            SqlValue::I64(v) => query.bind(v),
            SqlValue::F64(v) => query.bind(v),
            SqlValue::Bool(v) => query.bind(v),
            SqlValue::Date(v) => query.bind(v),
            SqlValue::DateTime(v) => query.bind(v),
            SqlValue::Null => query.bind(None::<String>),
        };
    }

    let result = query.execute(pool).await?;
    Ok(result.rows_affected())
}
