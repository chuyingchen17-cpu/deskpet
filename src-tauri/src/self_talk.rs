use chrono::{Local, Timelike, Utc};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};

use crate::state::RuntimeFlags;

pub async fn maybe_emit(app: &AppHandle, pool: &SqlitePool, flags: &RuntimeFlags) -> anyhow::Result<()> {
    if flags.do_not_disturb || !flags.self_talk_enabled {
        return Ok(());
    }

    if is_quiet_hours() {
        return Ok(());
    }

    let latest: Option<(chrono::DateTime<Utc>,)> =
        sqlx::query_as("SELECT created_at FROM self_talk_logs ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;

    if let Some((last_time,)) = latest {
        let elapsed = Utc::now() - last_time;
        if elapsed.num_minutes() < 30 {
            return Ok(());
        }
    }

    let content = "我刚刚看了下进度，你要不要先完成一个 10 分钟的小任务？";
    app.emit("self_talk_message", serde_json::json!({ "content": content }))?;

    sqlx::query("INSERT INTO self_talk_logs (id, content, created_at) VALUES (?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(content)
        .bind(Utc::now())
        .execute(pool)
        .await?;

    Ok(())
}

fn is_quiet_hours() -> bool {
    let now = Local::now();
    let hour = now.hour();
    hour >= 23 || hour < 8
}
