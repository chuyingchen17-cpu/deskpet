type ControlsProps = {
  dnd: boolean;
  selfTalk: boolean;
  systemControl: boolean;
  onToggleDnd: (next: boolean) => Promise<void>;
  onToggleSelfTalk: (next: boolean) => Promise<void>;
  onToggleSystemControl: (next: boolean) => Promise<void>;
  onQuickAction: () => Promise<void>;
};

export function Controls({
  dnd,
  selfTalk,
  systemControl,
  onToggleDnd,
  onToggleSelfTalk,
  onToggleSystemControl,
  onQuickAction
}: ControlsProps) {
  return (
    <section className="card controls">
      <h2>开关</h2>
      <label>
        <input type="checkbox" checked={dnd} onChange={(e) => onToggleDnd(e.target.checked)} />
        免打扰
      </label>
      <label>
        <input type="checkbox" checked={selfTalk} onChange={(e) => onToggleSelfTalk(e.target.checked)} />
        自言自语
      </label>
      <label>
        <input
          type="checkbox"
          checked={systemControl}
          onChange={(e) => onToggleSystemControl(e.target.checked)}
        />
        允许电脑操作
      </label>
      <button onClick={onQuickAction}>演示：打开浏览器</button>
    </section>
  );
}
