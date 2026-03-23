import { invoke } from '@tauri-apps/api/core';
import type { ChatResponse, TodoItem, Persona } from './types';

export const api = {
  sendChat: (message: string, sessionId: string, personaId: string) =>
    invoke<ChatResponse>('chat_send', {
      req: {
        message,
        session_id: sessionId,
        persona_id: personaId,
        context_mode: 'default'
      }
    }),

  listTodos: () => invoke<TodoItem[]>('todo_list'),

  createTodo: (title: string, dueAt?: string) =>
    invoke<TodoItem>('todo_create', {
      req: {
        title,
        due_at: dueAt ? new Date(dueAt).toISOString() : null,
        repeat_rule: null,
        priority: 2,
        source: 'ui'
      }
    }),

  completeTodo: (id: string) => invoke<void>('todo_complete', { id }),
  
  updateTodo: (id: string, title?: string, dueAt?: string | null) => {
    const req: Record<string, unknown> = {
      id
    };

    // Only include title if it's provided
    if (title !== undefined) {
      req.title = title;
    }

    // Handle due_at with explicit clear flag
    if (dueAt === null) {
      req.clear_due_at = true;
    } else if (typeof dueAt === 'string') {
      req.due_at = new Date(dueAt).toISOString();
      req.clear_due_at = false;
    }

    return invoke<TodoItem>('todo_update', { req });
  },
  
  deleteTodo: (id: string) => invoke<void>('todo_delete', { id }),

  listPersonas: () => invoke<Persona[]>('persona_list'),

  setSelfTalkEnabled: (enabled: boolean) => invoke<void>('self_talk_set_enabled', { enabled }),

  setDoNotDisturb: (enabled: boolean) => invoke<void>('set_do_not_disturb', { enabled }),

  setSystemControlEnabled: (enabled: boolean) => invoke<void>('system_control_set_enabled', { enabled }),

  executeSystemAction: (actionId: string, params: Record<string, string>, confirmed: boolean) =>
    invoke('system_action_execute', {
      req: {
        action_id: actionId,
        params,
        confirmed
      }
    }),

  panelOpen: (tab?: string) => invoke<void>('panel_open', { tab: tab ?? null }),
  panelHide: () => invoke<void>('panel_hide'),
  panelToggle: () => invoke<void>('panel_toggle'),
  appQuit: () => invoke<void>('app_quit'),
  snoozeReminder: (todoId: string, minutes = 10) =>
    invoke<void>('reminder_snooze', {
      req: {
        todo_id: todoId,
        minutes
      }
    }),

  dismissReminder: (todoId: string) =>
    invoke<void>('reminder_dismiss', {
      req: {
        todo_id: todoId
      }
    })
};
