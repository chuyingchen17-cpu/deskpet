import { FormEvent, useState } from 'react';
import type { TodoItem } from '../types';

type TodoPanelProps = {
  todos: TodoItem[];
  onCreate: (title: string, dueAt?: string) => Promise<void>;
  onComplete: (id: string) => Promise<void>;
  onUpdate: (id: string, title: string, dueAt?: string | null) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
};

export function TodoPanel({ todos, onCreate, onComplete, onUpdate, onDelete }: TodoPanelProps) {
  const [title, setTitle] = useState('');
  const [dueAt, setDueAt] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  const [editingDueAt, setEditingDueAt] = useState('');

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const t = title.trim();
    if (!t) return;
    await onCreate(t, dueAt || undefined);
    setTitle('');
    setDueAt('');
  };

  const startEdit = (todo: TodoItem) => {
    setEditingId(todo.id);
    setEditingTitle(todo.title);
    setEditingDueAt(todo.due_at ? new Date(todo.due_at).toISOString().slice(0, 16) : '');
  };

  const submitEdit = async (todoId: string) => {
    const t = editingTitle.trim();
    if (!t) return;
    await onUpdate(todoId, t, editingDueAt ? editingDueAt : null);
    setEditingId(null);
    setEditingTitle('');
    setEditingDueAt('');
  };

  return (
    <section className="card">
      <h2>待办</h2>
      <form onSubmit={submit} className="todo-form">
        <input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="新增待办" />
        <input type="datetime-local" value={dueAt} onChange={(e) => setDueAt(e.target.value)} />
        <button>添加</button>
      </form>
      <ul className="todo-list">
        {todos.map((todo) => (
          <li key={todo.id} className={todo.status === 'done' ? 'done' : ''}>
            <div>
              {editingId === todo.id ? (
                <div className="todo-form">
                  <input value={editingTitle} onChange={(e) => setEditingTitle(e.target.value)} />
                  <input
                    type="datetime-local"
                    value={editingDueAt}
                    onChange={(e) => setEditingDueAt(e.target.value)}
                  />
                </div>
              ) : (
                <>
                  <p>{todo.title}</p>
                  {todo.due_at && <small>提醒：{new Date(todo.due_at).toLocaleString()}</small>}
                </>
              )}
            </div>
            <div className="todo-actions">
              {todo.status === 'pending' ? <button onClick={() => onComplete(todo.id)}>完成</button> : <span>已完成</span>}
              {editingId === todo.id ? (
                <>
                  <button onClick={() => void submitEdit(todo.id)}>保存</button>
                  <button onClick={() => setEditingId(null)}>取消</button>
                </>
              ) : (
                <button onClick={() => startEdit(todo)}>编辑</button>
              )}
              <button onClick={() => onDelete(todo.id)}>删除</button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
