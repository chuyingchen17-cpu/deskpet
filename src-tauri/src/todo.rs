use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{TodoCreateRequest, TodoItem, TodoUpdateRequest};

pub async fn list(pool: &SqlitePool) -> anyhow::Result<Vec<TodoItem>> {
    let todos = sqlx::query_as::<_, TodoItem>(
        "SELECT id, title, due_at, repeat_rule, priority, status, source FROM todos ORDER BY status ASC, due_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(todos)
}

pub async fn create(pool: &SqlitePool, req: TodoCreateRequest) -> anyhow::Result<TodoItem> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO todos (id, title, due_at, repeat_rule, priority, status, source, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.title)
    .bind(req.due_at)
    .bind(&req.repeat_rule)
    .bind(req.priority)
    .bind("pending")
    .bind(&req.source)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(pool)
    .await?;

    get(pool, &id).await
}

pub async fn complete(pool: &SqlitePool, id: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE todos SET status = 'done', updated_at = ? WHERE id = ?")
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update(pool: &SqlitePool, req: TodoUpdateRequest) -> anyhow::Result<TodoItem> {
    let current = get(pool, &req.id).await?;
    let title = req.title.unwrap_or(current.title);
    let due_at = if req.clear_due_at.unwrap_or(false) {
        None
    } else {
        req.due_at.or(current.due_at)
    };
    let repeat_rule = req.repeat_rule.or(current.repeat_rule);
    let priority = req.priority.unwrap_or(current.priority);

    sqlx::query("UPDATE todos SET title = ?, due_at = ?, repeat_rule = ?, priority = ?, updated_at = ? WHERE id = ?")
        .bind(title)
        .bind(due_at)
        .bind(repeat_rule)
        .bind(priority)
        .bind(Utc::now())
        .bind(&req.id)
        .execute(pool)
        .await?;

    get(pool, &req.id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn get(pool: &SqlitePool, id: &str) -> anyhow::Result<TodoItem> {
    let todo = sqlx::query_as::<_, TodoItem>(
        "SELECT id, title, due_at, repeat_rule, priority, status, source FROM todos WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(todo)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sqlx::SqlitePool;

    use crate::models::{TodoCreateRequest, TodoUpdateRequest};

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should initialize");
        sqlx::query(
            "CREATE TABLE todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                due_at TEXT,
                repeat_rule TEXT,
                priority INTEGER NOT NULL DEFAULT 2,
                status TEXT NOT NULL DEFAULT 'pending',
                source TEXT NOT NULL,
                reminder_sent INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("todos table should be created");
        pool
    }

    #[tokio::test]
    async fn update_can_clear_due_at() {
        let pool = setup_pool().await;
        let due_at = Utc::now() + Duration::minutes(30);

        let created = super::create(
            &pool,
            TodoCreateRequest {
                title: "test".to_string(),
                due_at: Some(due_at),
                repeat_rule: None,
                priority: 2,
                source: "test".to_string(),
            },
        )
        .await
        .expect("todo create should succeed");

        let updated = super::update(
            &pool,
            TodoUpdateRequest {
                id: created.id,
                title: Some("updated".to_string()),
                due_at: None,
                clear_due_at: Some(true),
                repeat_rule: None,
                priority: None,
            },
        )
        .await
        .expect("todo update should succeed");

        assert_eq!(updated.title, "updated");
        assert!(updated.due_at.is_none());
    }

    #[tokio::test]
    async fn update_keeps_due_at_when_not_clearing() {
        let pool = setup_pool().await;
        let due_at = Utc::now() + Duration::minutes(45);

        let created = super::create(
            &pool,
            TodoCreateRequest {
                title: "test".to_string(),
                due_at: Some(due_at),
                repeat_rule: None,
                priority: 2,
                source: "test".to_string(),
            },
        )
        .await
        .expect("todo create should succeed");

        let updated = super::update(
            &pool,
            TodoUpdateRequest {
                id: created.id,
                title: Some("updated".to_string()),
                due_at: None,
                clear_due_at: Some(false),
                repeat_rule: None,
                priority: None,
            },
        )
        .await
        .expect("todo update should succeed");

        assert_eq!(updated.title, "updated");
        assert!(updated.due_at.is_some());
    }
}
