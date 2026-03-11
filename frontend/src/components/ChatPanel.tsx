import { FormEvent, useState } from 'react';

type ChatPanelProps = {
  onSend: (message: string) => Promise<void>;
  messages: Array<{ role: 'user' | 'pet'; text: string }>;
};

export function ChatPanel({ onSend, messages }: ChatPanelProps) {
  const [draft, setDraft] = useState('');
  const [sending, setSending] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const content = draft.trim();
    if (!content) return;
    setSending(true);
    try {
      await onSend(content);
      setDraft('');
    } finally {
      setSending(false);
    }
  };

  return (
    <section className="card">
      <h2>对话</h2>
      <div className="messages">
        {messages.map((msg, idx) => (
          <div key={`${msg.role}-${idx}`} className={`msg msg-${msg.role}`}>
            <strong>{msg.role === 'user' ? '你' : '宠物'}：</strong>
            {msg.text}
          </div>
        ))}
      </div>
      <form onSubmit={submit} className="input-row">
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="和桌宠聊点什么..."
          disabled={sending}
        />
        <button disabled={sending}>{sending ? '发送中' : '发送'}</button>
      </form>
    </section>
  );
}
