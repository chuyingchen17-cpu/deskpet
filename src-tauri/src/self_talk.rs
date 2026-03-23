use chrono::{Local, NaiveTime, Timelike, Utc};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};

use crate::state::RuntimeFlags;

pub async fn maybe_emit(app: &AppHandle, pool: &SqlitePool, flags: &RuntimeFlags) -> anyhow::Result<()> {
    if flags.do_not_disturb || !flags.self_talk_enabled {
        return Ok(());
    }

    // Load persona to get quiet hours
    let persona: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT quiet_hours_start, quiet_hours_end FROM personas WHERE id = 'default' LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    if is_quiet_hours(persona) {
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

fn is_quiet_hours(persona_hours: Option<(Option<String>, Option<String>)>) -> bool {
    let now = Local::now();
    let current_time = now.time();

    if let Some((Some(start_str), Some(end_str))) = persona_hours {
        // Parse quiet hours from persona settings
        if let (Ok(start), Ok(end)) = (
            NaiveTime::parse_from_str(&start_str, "%H:%M"),
            NaiveTime::parse_from_str(&end_str, "%H:%M"),
        ) {
            if start < end {
                // Normal case: e.g., 23:00 to 08:00 wraps around midnight
                return current_time >= start || current_time < end;
            } else {
                // Wrapping case: e.g., 08:00 to 23:00
                return current_time >= start && current_time < end;
            }
        }
    }

    // Fallback to default quiet hours if persona settings are invalid
    let hour = now.hour();
    hour >= 23 || hour < 8
}
