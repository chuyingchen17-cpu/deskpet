CREATE TABLE IF NOT EXISTS reminder_logs (
  id TEXT PRIMARY KEY,
  todo_id TEXT NOT NULL,
  event TEXT NOT NULL,
  title TEXT NOT NULL,
  due_at TEXT,
  source TEXT NOT NULL,
  created_at TEXT NOT NULL
);
