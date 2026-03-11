use sqlx::SqlitePool;

use crate::models::Persona;

pub async fn list(pool: &SqlitePool) -> anyhow::Result<Vec<Persona>> {
    let list = sqlx::query_as::<_, Persona>(
        "SELECT id, name, tone, style_tags, prohibited_topics, initiative_level, quiet_hours_start, quiet_hours_end FROM personas ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}
