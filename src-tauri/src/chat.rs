use anyhow::Context;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::{
    llm::LlmClient,
    models::{ChatRequest, ChatResponse, Persona},
};

pub async fn handle_chat(pool: &SqlitePool, llm: &LlmClient, req: ChatRequest) -> anyhow::Result<ChatResponse> {
    let persona: Persona = sqlx::query_as(
        "SELECT id, name, tone, style_tags, prohibited_topics, initiative_level, quiet_hours_start, quiet_hours_end FROM personas WHERE id = ?",
    )
    .bind(&req.persona_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("persona not found: {}", req.persona_id))?;

    let memory_summary = load_memory_summary(pool, &req.session_id).await?;

    let reply = llm
        .chat(&persona.name, &persona.tone, &memory_summary, &req.message)
        .await
        .unwrap_or_else(|err| {
            tracing::warn!("llm chat failed: {err:#}");
            "我刚才走神了，请再说一次。".to_string()
        });

    sqlx::query("INSERT INTO chat_messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&req.session_id)
        .bind("user")
        .bind(&req.message)
        .bind(Utc::now())
        .execute(pool)
        .await?;

    sqlx::query("INSERT INTO chat_messages (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&req.session_id)
        .bind("assistant")
        .bind(&reply)
        .bind(Utc::now())
        .execute(pool)
        .await?;

    let memory_update = format!("latest_context_mode={}", req.context_mode);
    sqlx::query("INSERT INTO memories (id, session_id, summary, created_at) VALUES (?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&req.session_id)
        .bind(&memory_update)
        .bind(Utc::now())
        .execute(pool)
        .await?;

    Ok(ChatResponse {
        reply,
        confidence: 0.82,
        actions: vec![],
        memory_updates: vec![memory_update],
    })
}

async fn load_memory_summary(pool: &SqlitePool, session_id: &str) -> anyhow::Result<String> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT summary FROM memories WHERE session_id = ? ORDER BY created_at DESC LIMIT 5",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .context("failed to load memory summary")?;

    if rows.is_empty() {
        return Ok("none".to_string());
    }

    Ok(rows
        .into_iter()
        .map(|(summary,)| summary)
        .collect::<Vec<_>>()
        .join("; "))
}
