use anyhow::Context;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;
use tauri::{AppHandle, Manager};

pub async fn init_pool(app: &AppHandle) -> anyhow::Result<SqlitePool> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(raw) = std::env::var("DESKPET_DB_PATH") {
        if !raw.trim().is_empty() {
            candidates.push(std::path::PathBuf::from(raw));
        }
    }

    if let Ok(app_dir) = app.path().app_data_dir() {
        candidates.push(app_dir.join("desktop_pet.db"));
    }

    let fallback_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".deskpet");
    candidates.push(fallback_dir.join("desktop_pet.db"));

    let mut last_err: Option<anyhow::Error> = None;
    for db_path in candidates {
        match try_open_pool(&db_path).await {
            Ok(pool) => {
                tracing::info!("sqlite ready at {}", db_path.display());
                return Ok(pool);
            }
            Err(err) => {
                tracing::warn!("failed to open sqlite at {}: {err:#}", db_path.display());
                last_err = Some(err);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("failed to initialize sqlite pool")))
}

async fn try_open_pool(db_path: &std::path::Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create sqlite directory")?;
    }
    let db_url = format!("sqlite://{}", db_path.display());
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("invalid sqlite url: {db_url}"))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .with_context(|| format!("failed to connect sqlite: {db_url}"))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    seed_default_persona(&pool).await?;
    Ok(pool)
}

async fn seed_default_persona(pool: &SqlitePool) -> anyhow::Result<()> {
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM personas WHERE id = ?")
        .bind("default")
        .fetch_optional(pool)
        .await?;

    if exists.is_none() {
        sqlx::query(
            "INSERT INTO personas (id, name, tone, style_tags, prohibited_topics, initiative_level, quiet_hours_start, quiet_hours_end)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("default")
        .bind("Claw Mini")
        .bind("friendly-pragmatic")
        .bind("efficiency,kind,focused")
        .bind("illegal,unsafe")
        .bind(60)
        .bind("23:00")
        .bind("08:00")
        .execute(pool)
        .await?;
    }

    Ok(())
}
