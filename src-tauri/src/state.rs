use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::llm::LlmClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFlags {
    pub do_not_disturb: bool,
    pub self_talk_enabled: bool,
    pub system_control_enabled: bool,
}

impl Default for RuntimeFlags {
    fn default() -> Self {
        Self {
            do_not_disturb: false,
            self_talk_enabled: true,
            system_control_enabled: false,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub llm: LlmClient,
    pub flags: Arc<RwLock<RuntimeFlags>>,
}
