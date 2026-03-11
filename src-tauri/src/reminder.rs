use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tokio_cron_scheduler::{Job, JobScheduler};

struct SnoozeResult {
    title: String,
    next_due_at: chrono::DateTime<Utc>,
}

struct DismissResult {
    title: String,
}

pub async fn start_scheduler(app: AppHandle, pool: SqlitePool) -> anyhow::Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;
    let app_clone = app.clone();
    let pool_clone = pool.clone();

    let job = Job::new_async("1/30 * * * * *", move |_uuid, _l| {
        let app = app_clone.clone();
        let pool = pool_clone.clone();
        Box::pin(async move {
            if let Err(err) = fire_due_reminders(&app, &pool).await {
                tracing::error!("failed to fire reminders: {err:#}");
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(scheduler)
}

async fn fire_due_reminders(app: &AppHandle, pool: &SqlitePool) -> anyhow::Result<()> {
    let rows: Vec<(String, String, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, title, due_at FROM todos WHERE status = 'pending' AND due_at IS NOT NULL AND due_at <= ? AND reminder_sent = 0",
    )
    .bind(Utc::now())
    .fetch_all(pool)
    .await?;

    for (id, title, due_at) in rows {
        let _ = app
            .notification()
            .builder()
            .title("桌宠提醒")
            .body(&title)
            .show();

        let _ = app.emit(
            "reminder_triggered",
            serde_json::json!({
                "todo_id": id,
                "title": title
            }),
        );

        insert_log(pool, &id, "triggered", &title, Some(due_at), "scheduler").await?;

        sqlx::query("UPDATE todos SET reminder_sent = 1, updated_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn snooze(
    app: &AppHandle,
    pool: &SqlitePool,
    todo_id: &str,
    minutes: i64,
) -> anyhow::Result<()> {
    let data = snooze_in_db(pool, todo_id, minutes).await?;
    app.emit(
        "reminder_snoozed",
        serde_json::json!({
            "todo_id": todo_id,
            "title": data.title,
            "next_due_at": data.next_due_at
        }),
    )?;
    Ok(())
}

pub async fn dismiss(app: &AppHandle, pool: &SqlitePool, todo_id: &str) -> anyhow::Result<()> {
    let data = dismiss_in_db(pool, todo_id).await?;
    app.emit(
        "reminder_dismissed",
        serde_json::json!({
            "todo_id": todo_id,
            "title": data.title
        }),
    )?;
    Ok(())
}

async fn snooze_in_db(pool: &SqlitePool, todo_id: &str, minutes: i64) -> anyhow::Result<SnoozeResult> {
    let duration_minutes = minutes.clamp(1, 720);
    let next_due_at = Utc::now() + Duration::minutes(duration_minutes);
    let row: Option<(String,)> = sqlx::query_as("SELECT title FROM todos WHERE id = ?")
        .bind(todo_id)
        .fetch_optional(pool)
        .await?;
    let Some((title,)) = row else {
        anyhow::bail!("todo not found: {todo_id}");
    };

    sqlx::query("UPDATE todos SET due_at = ?, reminder_sent = 0, updated_at = ? WHERE id = ?")
        .bind(next_due_at)
        .bind(Utc::now())
        .bind(todo_id)
        .execute(pool)
        .await?;

    insert_log(pool, todo_id, "snoozed", &title, Some(next_due_at), "user").await?;
    Ok(SnoozeResult { title, next_due_at })
}

async fn dismiss_in_db(pool: &SqlitePool, todo_id: &str) -> anyhow::Result<DismissResult> {
    let row: Option<(String, Option<chrono::DateTime<Utc>>)> =
        sqlx::query_as("SELECT title, due_at FROM todos WHERE id = ?")
            .bind(todo_id)
            .fetch_optional(pool)
            .await?;
    let Some((title, due_at)) = row else {
        anyhow::bail!("todo not found: {todo_id}");
    };

    sqlx::query("UPDATE todos SET reminder_sent = 1, updated_at = ? WHERE id = ?")
        .bind(Utc::now())
        .bind(todo_id)
        .execute(pool)
        .await?;

    insert_log(pool, todo_id, "dismissed", &title, due_at, "user").await?;
    Ok(DismissResult { title })
}

async fn insert_log(
    pool: &SqlitePool,
    todo_id: &str,
    event: &str,
    title: &str,
    due_at: Option<chrono::DateTime<Utc>>,
    source: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO reminder_logs (id, todo_id, event, title, due_at, source, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(todo_id)
    .bind(event)
    .bind(title)
    .bind(due_at)
    .bind(source)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sqlx::SqlitePool;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should initialize");
        sqlx::query(
            "CREATE TABLE todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                due_at TEXT,
                repeat_rule TEXT,
                priority INTEGER NOT NULL DEFAULT 2,
                status TEXT NOT NULL DEFAULT 'pending',
                source TEXT NOT NULL,
                reminder_sent INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("todos table should be created");
        sqlx::query(
            "CREATE TABLE reminder_logs (
                id TEXT PRIMARY KEY,
                todo_id TEXT NOT NULL,
                event TEXT NOT NULL,
                title TEXT NOT NULL,
                due_at TEXT,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("reminder_logs table should be created");
        pool
    }

    async fn seed_todo(
        pool: &SqlitePool,
        id: &str,
        title: &str,
        due_at: chrono::DateTime<Utc>,
        reminder_sent: i64,
    ) {
        sqlx::query(
            "INSERT INTO todos (id, title, due_at, repeat_rule, priority, status, source, reminder_sent, created_at, updated_at)
             VALUES (?, ?, ?, NULL, 2, 'pending', 'test', ?, ?, ?)",
        )
        .bind(id)
        .bind(title)
        .bind(due_at)
        .bind(reminder_sent)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(pool)
        .await
        .expect("seed todo should succeed");
    }

    #[tokio::test]
    async fn snooze_updates_due_at_resets_flag_and_writes_log() {
        let pool = setup_pool().await;
        let original_due_at = Utc::now() + Duration::minutes(5);
        seed_todo(&pool, "todo-1", "喝水", original_due_at, 1).await;

        let result = super::snooze_in_db(&pool, "todo-1", 10)
            .await
            .expect("snooze should succeed");

        let todo_row: (chrono::DateTime<Utc>, i64) =
            sqlx::query_as("SELECT due_at, reminder_sent FROM todos WHERE id = ?")
                .bind("todo-1")
                .fetch_one(&pool)
                .await
                .expect("todo row should exist");
        assert!(todo_row.0 > original_due_at);
        assert_eq!(todo_row.1, 0);
        assert_eq!(result.title, "喝水");

        let log_row: (String, String, String, Option<chrono::DateTime<Utc>>) = sqlx::query_as(
            "SELECT todo_id, event, source, due_at FROM reminder_logs ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("log row should exist");
        assert_eq!(log_row.0, "todo-1");
        assert_eq!(log_row.1, "snoozed");
        assert_eq!(log_row.2, "user");
        assert_eq!(log_row.3, Some(result.next_due_at));
    }

    #[tokio::test]
    async fn dismiss_marks_sent_and_writes_log() {
        let pool = setup_pool().await;
        let due_at = Utc::now() + Duration::minutes(12);
        seed_todo(&pool, "todo-2", "看周报", due_at, 0).await;

        let result = super::dismiss_in_db(&pool, "todo-2")
            .await
            .expect("dismiss should succeed");

        let reminder_sent: (i64,) = sqlx::query_as("SELECT reminder_sent FROM todos WHERE id = ?")
            .bind("todo-2")
            .fetch_one(&pool)
            .await
            .expect("todo row should exist");
        assert_eq!(reminder_sent.0, 1);
        assert_eq!(result.title, "看周报");

        let log_row: (String, String, String) = sqlx::query_as(
            "SELECT todo_id, event, source FROM reminder_logs ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("log row should exist");
        assert_eq!(log_row.0, "todo-2");
        assert_eq!(log_row.1, "dismissed");
        assert_eq!(log_row.2, "user");
    }
}
