use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;
use tokio::process::Command;

use crate::models::{SystemActionRequest, SystemActionResult};

pub async fn execute(
    pool: &SqlitePool,
    req: SystemActionRequest,
    system_control_enabled: bool,
) -> anyhow::Result<SystemActionResult> {
    let audit_id = uuid::Uuid::new_v4().to_string();

    if !system_control_enabled {
        insert_audit(
            pool,
            &audit_id,
            &req.action_id,
            "blocked",
            "system_control_disabled",
            &req.params,
        )
        .await?;

        return Ok(SystemActionResult {
            risk_level: "high".to_string(),
            requires_confirm: true,
            audit_id,
            result: "rejected: system control is disabled".to_string(),
        });
    }

    let (risk, requires_confirm) = risk_of(&req.action_id);
    if requires_confirm && !req.confirmed {
        insert_audit(
            pool,
            &audit_id,
            &req.action_id,
            "blocked",
            "confirmation_required",
            &req.params,
        )
        .await?;

        return Ok(SystemActionResult {
            risk_level: risk.to_string(),
            requires_confirm,
            audit_id,
            result: "rejected: confirmation required".to_string(),
        });
    }

    let result = run_action(&req.action_id, &req.params).await;
    let (status, output) = match result {
        Ok(text) => ("ok", text),
        Err(err) => ("error", err.to_string()),
    };

    insert_audit(pool, &audit_id, &req.action_id, status, &output, &req.params).await?;

    Ok(SystemActionResult {
        risk_level: risk.to_string(),
        requires_confirm,
        audit_id,
        result: output,
    })
}

fn risk_of(action_id: &str) -> (&'static str, bool) {
    match action_id {
        "run_script" => ("high", true),
        "open_app" | "open_url" | "switch_app" => ("medium", false),
        _ => ("high", true),
    }
}

async fn run_action(action_id: &str, params: &Value) -> anyhow::Result<String> {
    match action_id {
        "open_app" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("Safari");
            ensure_success(
                Command::new("open").arg("-a").arg(app).output().await?,
                "open_app",
            )?;
            Ok(format!("opened app: {app}"))
        }
        "open_url" => {
            let url = params
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://www.apple.com");
            ensure_success(Command::new("open").arg(url).output().await?, "open_url")?;
            Ok(format!("opened url: {url}"))
        }
        "switch_app" => {
            let app = params.get("app").and_then(|v| v.as_str()).unwrap_or("Safari");
            let script = format!("tell application \"{app}\" to activate");
            ensure_success(
                Command::new("osascript").arg("-e").arg(script).output().await?,
                "switch_app",
            )?;
            Ok(format!("switched to app: {app}"))
        }
        "run_script" => {
            let script = params.get("script").and_then(|v| v.as_str()).unwrap_or("");
            if script.is_empty() {
                anyhow::bail!("missing script");
            }
            let output = Command::new("/bin/zsh").arg("-lc").arg(script).output().await?;
            ensure_success(output, "run_script")
        }
        _ => anyhow::bail!("unsupported action: {action_id}"),
    }
}

fn ensure_success(output: std::process::Output, action_id: &str) -> anyhow::Result<String> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        anyhow::bail!("{action_id} failed with status {}", output.status);
    }
    anyhow::bail!("{action_id} failed: {stderr}");
}

async fn insert_audit(
    pool: &SqlitePool,
    audit_id: &str,
    action_id: &str,
    status: &str,
    result: &str,
    params: &Value,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO action_audit_logs (id, action_id, status, result, params_json, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(audit_id)
    .bind(action_id)
    .bind(status)
    .bind(result)
    .bind(params.to_string())
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{os::unix::process::ExitStatusExt, process::Output};

    use serde_json::json;
    use sqlx::SqlitePool;

    use crate::models::SystemActionRequest;

    use super::{ensure_success, execute, risk_of};

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should initialize");
        sqlx::query(
            "CREATE TABLE action_audit_logs (
                id TEXT PRIMARY KEY,
                action_id TEXT NOT NULL,
                status TEXT NOT NULL,
                result TEXT NOT NULL,
                params_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("action_audit_logs table should be created");
        pool
    }

    #[test]
    fn run_script_requires_confirm() {
        let (risk, confirm) = risk_of("run_script");
        assert_eq!(risk, "high");
        assert!(confirm);
    }

    #[test]
    fn open_url_is_medium_risk() {
        let (risk, confirm) = risk_of("open_url");
        assert_eq!(risk, "medium");
        assert!(!confirm);
    }

    #[test]
    fn ensure_success_returns_stdout_on_zero_exit() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"ok\n".to_vec(),
            stderr: vec![],
        };
        let result = ensure_success(output, "run_script").expect("status 0 should pass");
        assert_eq!(result, "ok");
    }

    #[test]
    fn ensure_success_returns_error_on_non_zero_exit() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(256),
            stdout: vec![],
            stderr: b"permission denied\n".to_vec(),
        };
        let err = ensure_success(output, "run_script").expect_err("non-zero status should fail");
        let message = err.to_string();
        assert!(message.contains("run_script failed"));
        assert!(message.contains("permission denied"));
    }

    #[tokio::test]
    async fn execute_blocks_when_system_control_is_disabled() {
        let pool = setup_pool().await;
        let result = execute(
            &pool,
            SystemActionRequest {
                action_id: "open_url".to_string(),
                params: json!({ "url": "https://example.com" }),
                confirmed: true,
            },
            false,
        )
        .await
        .expect("execute should return blocked result");

        assert!(result.result.contains("system control is disabled"));
        assert!(result.requires_confirm);

        let row: (String, String) =
            sqlx::query_as("SELECT status, result FROM action_audit_logs ORDER BY created_at DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("audit row should exist");
        assert_eq!(row.0, "blocked");
        assert_eq!(row.1, "system_control_disabled");
    }

    #[tokio::test]
    async fn execute_blocks_run_script_without_confirmation() {
        let pool = setup_pool().await;
        let result = execute(
            &pool,
            SystemActionRequest {
                action_id: "run_script".to_string(),
                params: json!({ "script": "echo hello" }),
                confirmed: false,
            },
            true,
        )
        .await
        .expect("execute should return blocked result");

        assert!(result.result.contains("confirmation required"));
        assert!(result.requires_confirm);
        assert_eq!(result.risk_level, "high");

        let row: (String, String) =
            sqlx::query_as("SELECT status, result FROM action_audit_logs ORDER BY created_at DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("audit row should exist");
        assert_eq!(row.0, "blocked");
        assert_eq!(row.1, "confirmation_required");
    }
}
