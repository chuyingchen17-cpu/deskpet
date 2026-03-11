# IPC API Contract (Tauri Commands)

## Chat

- `chat_send(req)`
- req:
  - `message: string`
  - `session_id: string`
  - `persona_id: string`
  - `context_mode: string`
- resp:
  - `reply: string`
  - `confidence: number`
  - `actions: string[]`
  - `memory_updates: string[]`

## Todo

- `todo_list()` -> `TodoItem[]`
- `todo_create(req)` -> `TodoItem`
- `todo_complete(id)` -> `void`
- `todo_update(req)` -> `TodoItem`
  - `req.clear_due_at?: boolean` (`true` 时清空提醒时间)
- `todo_delete(id)` -> `void`

## Persona

- `persona_list()` -> `Persona[]`

## Runtime Flags

- `set_do_not_disturb(enabled)`
- `self_talk_set_enabled(enabled)`
- `system_control_set_enabled(enabled)`

## Window Control

- `panel_open(tab?: string)`
- `panel_hide()`
- `panel_toggle()`
- `app_quit()`

## Reminder

- `reminder_snooze(req)` -> `void`
- req:
  - `todo_id: string`
  - `minutes: number`
- `reminder_dismiss(req)` -> `void`
- req:
  - `todo_id: string`

## System Action

- `system_action_execute(req)`
- req:
  - `action_id: "open_app" | "open_url" | "switch_app" | "run_script"`
  - `params: object`
  - `confirmed: boolean`
- resp:
  - `risk_level: string`
  - `requires_confirm: boolean`
  - `audit_id: string`
  - `result: string`

## Events

- `reminder_triggered`
- `reminder_snoozed`
- `reminder_dismissed`
- `self_talk_message`
- `panel_tab_changed`
