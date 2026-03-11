use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub session_id: String,
    pub persona_id: String,
    pub context_mode: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
    pub confidence: f32,
    pub actions: Vec<String>,
    pub memory_updates: Vec<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub due_at: Option<DateTime<Utc>>,
    pub repeat_rule: Option<String>,
    pub priority: i32,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct TodoCreateRequest {
    pub title: String,
    pub due_at: Option<DateTime<Utc>>,
    pub repeat_rule: Option<String>,
    pub priority: i32,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct TodoUpdateRequest {
    pub id: String,
    pub title: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub clear_due_at: Option<bool>,
    pub repeat_rule: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReminderSnoozeRequest {
    pub todo_id: String,
    pub minutes: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReminderDismissRequest {
    pub todo_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub tone: String,
    pub style_tags: String,
    pub prohibited_topics: String,
    pub initiative_level: i32,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SystemActionRequest {
    pub action_id: String,
    pub params: serde_json::Value,
    pub confirmed: bool,
}

#[derive(Debug, Serialize)]
pub struct SystemActionResult {
    pub risk_level: String,
    pub requires_confirm: bool,
    pub audit_id: String,
    pub result: String,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub db_ready: bool,
    pub llm_mode: String,
}
