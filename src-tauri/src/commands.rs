use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    chat,
    models::{
        ChatRequest, ChatResponse, HealthStatus, Persona, SystemActionRequest, SystemActionResult,
        ReminderDismissRequest, ReminderSnoozeRequest, TodoCreateRequest, TodoItem, TodoUpdateRequest,
    },
    persona, reminder, self_talk, system_action, todo,
    state::AppState,
};

#[tauri::command]
pub async fn health(state: State<'_, AppState>) -> Result<HealthStatus, String> {
    Ok(HealthStatus {
        status: "ok".to_string(),
        db_ready: true,
        llm_mode: state.llm.mode().to_string(),
    })
}

#[tauri::command]
pub async fn chat_send(state: State<'_, AppState>, req: ChatRequest) -> Result<ChatResponse, String> {
    let pool = state.pool.clone();
    let llm = state.llm.clone();

    let response = chat::handle_chat(&pool, &llm, req)
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
pub async fn todo_list(state: State<'_, AppState>) -> Result<Vec<TodoItem>, String> {
    todo::list(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn todo_create(state: State<'_, AppState>, req: TodoCreateRequest) -> Result<TodoItem, String> {
    todo::create(&state.pool, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn todo_complete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    todo::complete(&state.pool, &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn todo_update(state: State<'_, AppState>, req: TodoUpdateRequest) -> Result<TodoItem, String> {
    todo::update(&state.pool, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn todo_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    todo::delete(&state.pool, &id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn persona_list(state: State<'_, AppState>) -> Result<Vec<Persona>, String> {
    persona::list(&state.pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_do_not_disturb(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    {
        let mut flags = state.flags.write().await;
        flags.do_not_disturb = enabled;
    }
    Ok(())
}

#[tauri::command]
pub async fn self_talk_set_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    {
        let mut flags = state.flags.write().await;
        flags.self_talk_enabled = enabled;
    }
    Ok(())
}

#[tauri::command]
pub async fn system_control_set_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    {
        let mut flags = state.flags.write().await;
        flags.system_control_enabled = enabled;
    }
    Ok(())
}

#[tauri::command]
pub async fn system_action_execute(
    state: State<'_, AppState>,
    req: SystemActionRequest,
) -> Result<SystemActionResult, String> {
    let flags = state.flags.read().await;
    system_action::execute(&state.pool, req, flags.system_control_enabled)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn panel_open(app: AppHandle, tab: Option<String>) -> Result<(), String> {
    let panel = app
        .get_webview_window("panel")
        .ok_or_else(|| "panel window not found".to_string())?;
    panel.show().map_err(|e| e.to_string())?;
    panel.set_focus().map_err(|e| e.to_string())?;

    if let Some(tab_name) = tab {
        panel
            .emit(
                "panel_tab_changed",
                serde_json::json!({
                    "tab": tab_name
                }),
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn panel_hide(app: AppHandle) -> Result<(), String> {
    let panel = app
        .get_webview_window("panel")
        .ok_or_else(|| "panel window not found".to_string())?;
    panel.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn panel_toggle(app: AppHandle) -> Result<(), String> {
    let panel = app
        .get_webview_window("panel")
        .ok_or_else(|| "panel window not found".to_string())?;
    let visible = panel.is_visible().map_err(|e| e.to_string())?;
    if visible {
        panel.hide().map_err(|e| e.to_string())?;
    } else {
        panel.show().map_err(|e| e.to_string())?;
        panel.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn app_quit(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn reminder_snooze(
    app: AppHandle,
    state: State<'_, AppState>,
    req: ReminderSnoozeRequest,
) -> Result<(), String> {
    reminder::snooze(&app, &state.pool, &req.todo_id, req.minutes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reminder_dismiss(
    app: AppHandle,
    state: State<'_, AppState>,
    req: ReminderDismissRequest,
) -> Result<(), String> {
    reminder::dismiss(&app, &state.pool, &req.todo_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn start_background_jobs(app: AppHandle) -> anyhow::Result<()> {
    let state = app.state::<AppState>();
    let scheduler = reminder::start_scheduler(app.clone(), state.pool.clone()).await?;

    let self_talk_app = app.clone();
    let self_talk_pool = state.pool.clone();
    let self_talk_flags = state.flags.clone();

    tauri::async_runtime::spawn(async move {
        let _scheduler = scheduler;
        loop {
            // Clone flags immediately to minimize lock duration
            let snapshot = {
                let flags = self_talk_flags.read().await;
                flags.clone()
            };
            
            if let Err(err) = self_talk::maybe_emit(&self_talk_app, &self_talk_pool, &snapshot).await {
                tracing::warn!("self talk emit failed: {err:#}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(120)).await;
        }
    });

    Ok(())
}
