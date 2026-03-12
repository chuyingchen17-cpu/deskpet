import { useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { PhysicalPosition, currentMonitor, getCurrentWindow } from '@tauri-apps/api/window';
import { api } from './api';
import { PetAvatar } from './components/PetAvatar';
import { ChatPanel } from './components/ChatPanel';
import { TodoPanel } from './components/TodoPanel';
import { Controls } from './components/Controls';
import { useSessionId } from './hooks/useSession';
import type { TodoItem, Persona } from './types';

type PanelTab = 'menu' | 'chat' | 'todo' | 'controls' | 'account' | 'api';

const PANEL_TABS: PanelTab[] = ['menu', 'chat', 'todo', 'controls', 'account', 'api'];

function toPanelTab(value: string): PanelTab | null {
  return PANEL_TABS.includes(value as PanelTab) ? (value as PanelTab) : null;
}

export function App() {
  const sessionId = useSessionId();
  const snapTimerRef = useRef<number | null>(null);
  const snappingRef = useRef(false);
  const [windowLabel, setWindowLabel] = useState<string>('main');
  const [tab, setTab] = useState<PanelTab>('menu');
  const [hovering, setHovering] = useState(false);

  const [messages, setMessages] = useState<Array<{ role: 'user' | 'pet'; text: string }>>([
    { role: 'pet', text: '你好，我是 Claw Mini。点击上方菜单进入功能。' }
  ]);
  const [todos, setTodos] = useState<TodoItem[]>([]);
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [dnd, setDnd] = useState(false);
  const [selfTalk, setSelfTalk] = useState(true);
  const [systemControl, setSystemControl] = useState(false);
  const [speaking, setSpeaking] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [activeReminder, setActiveReminder] = useState<{ todoId: string; title: string } | null>(null);

  const [accountName, setAccountName] = useState(() => localStorage.getItem('deskpet_account_name') ?? 'Deskpet User');
  const [accountEmail, setAccountEmail] = useState(() => localStorage.getItem('deskpet_account_email') ?? 'user@example.com');
  const [apiKey, setApiKey] = useState(() => localStorage.getItem('deskpet_api_key') ?? '');
  const [apiModel, setApiModel] = useState(() => localStorage.getItem('deskpet_api_model') ?? 'gpt-4o-mini');

  const isPanelWindow = windowLabel === 'panel';
  const activePersonaId = personas[0]?.id ?? 'default';

  useEffect(() => {
    setWindowLabel(getCurrentWindow().label);
  }, []);

  useEffect(() => {
    if (isPanelWindow) {
      document.body.classList.remove('pet-mode');
      return;
    }
    document.body.classList.add('pet-mode');
    return () => document.body.classList.remove('pet-mode');
  }, [isPanelWindow]);

  useEffect(() => {
    if (isPanelWindow) return;

    const win = getCurrentWindow();
    let disposed = false;

    const setup = async () => {
      const unlisten = await win.onMoved(() => {
        if (snappingRef.current) return;
        if (snapTimerRef.current !== null) {
          window.clearTimeout(snapTimerRef.current);
        }

        snapTimerRef.current = window.setTimeout(async () => {
          if (disposed || snappingRef.current) return;
          snappingRef.current = true;
          try {
            const [position, size, monitor] = await Promise.all([
              win.outerPosition(),
              win.outerSize(),
              currentMonitor()
            ]);
            if (!monitor) return;

            const margin = 8;
            const minX = monitor.position.x + margin;
            const minY = monitor.position.y + margin;
            const maxX = monitor.position.x + monitor.size.width - size.width - margin;
            const maxY = monitor.position.y + monitor.size.height - size.height - margin;

            const x = Math.min(Math.max(position.x, minX), maxX);
            const y = Math.min(Math.max(position.y, minY), maxY);

            const distances = [
              { edge: 'left', value: Math.abs(x - minX) },
              { edge: 'right', value: Math.abs(maxX - x) },
              { edge: 'top', value: Math.abs(y - minY) },
              { edge: 'bottom', value: Math.abs(maxY - y) }
            ];
            distances.sort((a, b) => a.value - b.value);

            let snapX = x;
            let snapY = y;
            switch (distances[0].edge) {
              case 'left':
                snapX = minX;
                break;
              case 'right':
                snapX = maxX;
                break;
              case 'top':
                snapY = minY;
                break;
              case 'bottom':
                snapY = maxY;
                break;
            }

            await win.setPosition(new PhysicalPosition(Math.round(snapX), Math.round(snapY)));
          } finally {
            snappingRef.current = false;
          }
        }, 180);
      });

      return unlisten;
    };

    let unlistenFn: (() => void) | null = null;
    void setup().then((unlisten) => {
      unlistenFn = unlisten;
    });

    return () => {
      disposed = true;
      if (snapTimerRef.current !== null) {
        window.clearTimeout(snapTimerRef.current);
      }
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [isPanelWindow]);

  const refreshTodos = async () => {
    const list = await api.listTodos();
    setTodos(list);
  };

  const withError = async (action: () => Promise<void>, fallbackMessage: string) => {
    try {
      setErrorMessage(null);
      await action();
      return true;
    } catch (error) {
      const message = error instanceof Error && error.message ? error.message : fallbackMessage;
      setErrorMessage(message);
      return false;
    }
  };

  useEffect(() => {
    if (!isPanelWindow) return;

    void refreshTodos();
    void api.listPersonas().then(setPersonas).catch(() => setPersonas([]));

    const unlistenPromises = [
      listen<{ todo_id: string; title: string }>('reminder_triggered', (event) => {
        setActiveReminder({ todoId: event.payload.todo_id, title: event.payload.title });
        setMessages((prev) => [...prev, { role: 'pet', text: `提醒你：${event.payload.title}` }]);
      }),
      listen<{ todo_id: string; title: string; next_due_at: string }>('reminder_snoozed', (event) => {
        setActiveReminder(null);
        setMessages((prev) => [
          ...prev,
          { role: 'pet', text: `已稍后提醒：${event.payload.title}（${new Date(event.payload.next_due_at).toLocaleTimeString()}）` }
        ]);
      }),
      listen<{ todo_id: string; title: string }>('reminder_dismissed', (event) => {
        setActiveReminder(null);
        setMessages((prev) => [...prev, { role: 'pet', text: `已忽略本次提醒：${event.payload.title}` }]);
      }),
      listen<{ content: string }>('self_talk_message', (event) => {
        setMessages((prev) => [...prev, { role: 'pet', text: event.payload.content }]);
      }),
      listen<{ tab: string }>('panel_tab_changed', (event) => {
        const nextTab = toPanelTab(event.payload.tab);
        if (nextTab) setTab(nextTab);
      })
    ];

    return () => {
      void Promise.all(unlistenPromises).then((fns) => fns.forEach((fn) => fn()));
    };
  }, [isPanelWindow]);

  const sendMessage = async (text: string) => {
    setMessages((prev) => [...prev, { role: 'user', text }]);
    setSpeaking(true);
    try {
      const reply = await api.sendChat(text, sessionId, activePersonaId);
      setMessages((prev) => [...prev, { role: 'pet', text: reply.reply }]);
      await refreshTodos();
    } catch {
      setMessages((prev) => [...prev, { role: 'pet', text: '我刚刚分心了，请再说一次。' }]);
    } finally {
      setSpeaking(false);
    }
  };

  const menuActions = useMemo(
    () => [
      { label: '对话', tab: 'chat' as PanelTab },
      { label: '待办', tab: 'todo' as PanelTab },
      { label: '开关', tab: 'controls' as PanelTab },
      { label: '账户设置', tab: 'account' as PanelTab },
      { label: 'API 设置', tab: 'api' as PanelTab }
    ],
    []
  );

  const openPanelAt = async (targetTab: PanelTab) => {
    await api.panelOpen(targetTab);
  };

  const logout = () => {
    localStorage.removeItem('desktop_pet_session_id');
    window.location.reload();
  };

  if (!isPanelWindow) {
    return (
      <main className="pet-window">
        <div
          className="pet-shell"
          data-tauri-drag-region
          onMouseDown={(event) => {
            if (event.button !== 0) return;
            const target = event.target as HTMLElement;
            if (target.closest('.hover-menu')) return;
            void getCurrentWindow().startDragging();
          }}
          onMouseEnter={() => setHovering(true)}
          onMouseLeave={() => setHovering(false)}
        >
          <div
            className="pet-launcher"
            data-tauri-drag-region
            role="button"
            tabIndex={0}
            aria-label="打开菜单"
            onClick={() => {
              void api.panelOpen('menu');
            }}
            onKeyDown={(event) => {
              if (event.key === 'Enter' || event.key === ' ') {
                void api.panelOpen('menu');
              }
            }}
          >
            <PetAvatar speaking label="desktop pet" />
          </div>

          <aside className={`hover-menu ${hovering ? 'show' : ''}`} onMouseDown={(e) => e.stopPropagation()}>
            {menuActions.map((item) => (
              <button key={item.tab} onClick={() => void openPanelAt(item.tab)}>
                {item.label}
              </button>
            ))}
            <button
              className="danger-btn"
              onClick={() => {
                logout();
              }}
            >
              退出登录
            </button>
            <button
              className="danger-btn"
              onClick={() => {
                void api.appQuit();
              }}
            >
              退出应用
            </button>
          </aside>
        </div>
      </main>
    );
  }

  return (
    <main className="app panel-mode">
      <header className="header panel-header">
        <PetAvatar speaking={speaking} label="desktop pet" />
        <div>
          <h1>Claw Mini</h1>
          <p>点击菜单项后进入功能</p>
        </div>
        <button
          className="ghost-btn"
          onClick={() => {
            void api.panelHide();
          }}
        >
          收起
        </button>
      </header>

      <nav className="menu-bar">
        <button onClick={() => setTab('chat')}>对话</button>
        <button onClick={() => setTab('todo')}>待办</button>
        <button onClick={() => setTab('controls')}>开关</button>
        <button onClick={() => setTab('account')}>账户设置</button>
        <button onClick={() => setTab('api')}>API 设置</button>
      </nav>

      <div className="grid">
        {errorMessage && (
          <section className="card error-card" role="alert">
            <h2>操作失败</h2>
            <p>{errorMessage}</p>
          </section>
        )}

        {activeReminder && (
          <section className="card reminder-card">
            <h2>当前提醒</h2>
            <p>{activeReminder.title}</p>
            <div className="todo-actions">
              <button
                onClick={async () => {
                  await withError(async () => {
                    await api.snoozeReminder(activeReminder.todoId, 10);
                    await refreshTodos();
                  }, '提醒稍后失败，请重试。');
                }}
              >
                10 分钟后提醒
              </button>
              <button
                onClick={async () => {
                  await withError(async () => {
                    await api.dismissReminder(activeReminder.todoId);
                    await refreshTodos();
                  }, '忽略提醒失败，请重试。');
                }}
              >
                忽略
              </button>
            </div>
          </section>
        )}

        {tab === 'menu' && (
          <section className="card">
            <h2>菜单</h2>
            <p>请选择上方菜单项进入功能。</p>
          </section>
        )}

        {tab === 'chat' && <ChatPanel onSend={sendMessage} messages={messages} />}

        {tab === 'todo' && (
          <TodoPanel
            todos={todos}
            onCreate={async (title, dueAt) => {
              await withError(async () => {
                await api.createTodo(title, dueAt);
                await refreshTodos();
              }, '创建待办失败，请检查输入后重试。');
            }}
            onComplete={async (id) => {
              await withError(async () => {
                await api.completeTodo(id);
                await refreshTodos();
              }, '更新待办状态失败，请重试。');
            }}
            onUpdate={async (id, title, dueAt) => {
              await withError(async () => {
                await api.updateTodo(id, title, dueAt);
                await refreshTodos();
              }, '编辑待办失败，请重试。');
            }}
            onDelete={async (id) => {
              await withError(async () => {
                await api.deleteTodo(id);
                await refreshTodos();
              }, '删除待办失败，请重试。');
            }}
          />
        )}

        {tab === 'controls' && (
          <Controls
            dnd={dnd}
            selfTalk={selfTalk}
            systemControl={systemControl}
            onToggleDnd={async (next) => {
              const previous = dnd;
              setDnd(next);
              const ok = await withError(async () => {
                await api.setDoNotDisturb(next);
              }, '切换免打扰失败，请重试。');
              if (!ok) setDnd(previous);
            }}
            onToggleSelfTalk={async (next) => {
              const previous = selfTalk;
              setSelfTalk(next);
              const ok = await withError(async () => {
                await api.setSelfTalkEnabled(next);
              }, '切换自言自语失败，请重试。');
              if (!ok) setSelfTalk(previous);
            }}
            onToggleSystemControl={async (next) => {
              const previous = systemControl;
              setSystemControl(next);
              const ok = await withError(async () => {
                await api.setSystemControlEnabled(next);
              }, '切换系统控制失败，请重试。');
              if (!ok) setSystemControl(previous);
            }}
            onQuickAction={async () => {
              await withError(async () => {
                const response = await api.executeSystemAction('open_url', { url: 'https://www.apple.com' }, true);
                const text = String((response as { result: string }).result);
                setMessages((prev) => [...prev, { role: 'pet', text: `系统动作结果：${text}` }]);
                setTab('chat');
              }, '执行系统动作失败，请检查权限或设置。');
            }}
          />
        )}

        {tab === 'account' && (
          <section className="card">
            <h2>账户设置</h2>
            <div className="settings-form">
              <label>
                昵称
                <input value={accountName} onChange={(e) => setAccountName(e.target.value)} />
              </label>
              <label>
                邮箱
                <input value={accountEmail} onChange={(e) => setAccountEmail(e.target.value)} />
              </label>
              <div className="todo-actions">
                <button
                  onClick={() => {
                    localStorage.setItem('deskpet_account_name', accountName);
                    localStorage.setItem('deskpet_account_email', accountEmail);
                    setMessages((prev) => [...prev, { role: 'pet', text: '账户设置已保存。' }]);
                  }}
                >
                  保存设置
                </button>
                <button
                  className="danger-btn"
                  onClick={() => {
                    logout();
                  }}
                >
                  退出登录
                </button>
              </div>
            </div>
          </section>
        )}

        {tab === 'api' && (
          <section className="card">
            <h2>API 设置</h2>
            <div className="settings-form">
              <label>
                API Key
                <input type="password" value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder="sk-..." />
              </label>
              <label>
                Model
                <input value={apiModel} onChange={(e) => setApiModel(e.target.value)} placeholder="gpt-4o-mini" />
              </label>
              <p className="hint-text">该配置保存到本地。修改后请重启应用以生效。</p>
              <div className="todo-actions">
                <button
                  onClick={() => {
                    localStorage.setItem('deskpet_api_key', apiKey);
                    localStorage.setItem('deskpet_api_model', apiModel);
                    setMessages((prev) => [...prev, { role: 'pet', text: 'API 设置已保存，重启后生效。' }]);
                  }}
                >
                  保存 API 设置
                </button>
                <button
                  className="danger-btn"
                  onClick={() => {
                    void api.appQuit();
                  }}
                >
                  退出应用
                </button>
              </div>
            </div>
          </section>
        )}
      </div>
    </main>
  );
}
