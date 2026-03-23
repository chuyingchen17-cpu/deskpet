-- Add indexes for better query performance

-- Chat messages queries by session_id
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_id ON chat_messages(session_id, created_at DESC);

-- Memories queries by session_id
CREATE INDEX IF NOT EXISTS idx_memories_session_id ON memories(session_id, created_at DESC);

-- Todos queries by status and due_at
CREATE INDEX IF NOT EXISTS idx_todos_status_due_at ON todos(status, due_at);

-- Reminder logs queries by todo_id
CREATE INDEX IF NOT EXISTS idx_reminder_logs_todo_id ON reminder_logs(todo_id, created_at DESC);

-- Action audit logs queries by action_id
CREATE INDEX IF NOT EXISTS idx_action_audit_logs_action_id ON action_audit_logs(action_id, created_at DESC);

-- Self talk logs queries by created_at
CREATE INDEX IF NOT EXISTS idx_self_talk_logs_created_at ON self_talk_logs(created_at DESC);
