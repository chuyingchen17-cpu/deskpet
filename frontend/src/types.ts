export type TodoStatus = 'pending' | 'done';

export interface TodoItem {
  id: string;
  title: string;
  due_at?: string | null;
  repeat_rule?: string | null;
  priority: number;
  status: TodoStatus;
  source: string;
}

export interface ChatResponse {
  reply: string;
  confidence: number;
  actions: string[];
  memory_updates: string[];
}

export interface Persona {
  id: string;
  name: string;
  tone: string;
  style_tags: string;
  prohibited_topics: string;
  initiative_level: number;
  quiet_hours_start?: string | null;
  quiet_hours_end?: string | null;
}
