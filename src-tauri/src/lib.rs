mod chat;
mod commands;
mod db;
mod llm;
mod models;
mod persona;
mod reminder;
mod self_talk;
mod state;
mod system_action;
mod todo;

use state::{AppState, RuntimeFlags};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pool = db::init_pool(&app_handle).await?;
                let state = AppState {
                    pool,
                    llm: llm::LlmClient::new(),
                    flags: std::sync::Arc::new(tokio::sync::RwLock::new(RuntimeFlags::default())),
                };

                app_handle.manage(state);

                tauri::WebviewWindowBuilder::new(
                    &app_handle,
                    "panel",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("Desktop Pet Menu")
                .inner_size(420.0, 680.0)
                .resizable(true)
                .always_on_top(true)
                .visible(false)
                .build()?;

                commands::start_background_jobs(app_handle.clone()).await?;

                anyhow::Ok(())
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health,
            commands::chat_send,
            commands::todo_list,
            commands::todo_create,
            commands::todo_complete,
            commands::todo_update,
            commands::todo_delete,
            commands::persona_list,
            commands::set_do_not_disturb,
            commands::self_talk_set_enabled,
            commands::system_control_set_enabled,
            commands::system_action_execute,
            commands::panel_open,
            commands::panel_hide,
            commands::panel_toggle,
            commands::app_quit,
            commands::reminder_snooze,
            commands::reminder_dismiss
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
