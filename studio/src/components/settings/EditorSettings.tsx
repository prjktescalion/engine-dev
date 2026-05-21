import { useState } from 'react';
import { useSettingsStore } from '../../store/settingsStore';
import type { EditorSettings as Settings } from '../../types/engine';

interface EditorSettingsProps {
  onClose: () => void;
}

const EDITOR_LABELS: Record<Settings['editor'], string> = {
  vscode: 'VS Code  (code)',
  jetbrains: 'JetBrains  (idea)',
  custom: 'Custom binary',
};

export function EditorSettingsModal({ onClose }: EditorSettingsProps) {
  const { editor, custom_path, theme, save } = useSettingsStore();
  const [localEditor, setLocalEditor] = useState<Settings['editor']>(editor);
  const [localPath, setLocalPath] = useState(custom_path);
  const [localTheme, setLocalTheme] = useState<Settings['theme']>(theme);

  async function apply() {
    await save({ editor: localEditor, custom_path: localPath, theme: localTheme });
    onClose();
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal__header">
          <h2>Settings</h2>
          <button className="modal__close" onClick={onClose}>✕</button>
        </div>

        <div className="modal__body">
          <section className="settings-section">
            <label className="settings-section__label">Preferred Code Editor</label>
            {(Object.keys(EDITOR_LABELS) as Settings['editor'][]).map((key) => (
              <label key={key} className="radio-row">
                <input
                  type="radio"
                  name="editor"
                  value={key}
                  checked={localEditor === key}
                  onChange={() => setLocalEditor(key)}
                />
                {EDITOR_LABELS[key]}
              </label>
            ))}

            {localEditor === 'custom' && (
              <div className="settings-section__field">
                <label>Editor binary path</label>
                <input
                  type="text"
                  placeholder="/usr/local/bin/my-editor"
                  value={localPath}
                  onChange={(e) => setLocalPath(e.target.value)}
                />
              </div>
            )}
          </section>

          <section className="settings-section">
            <label className="settings-section__label">Theme</label>
            {(['dark', 'light'] as Settings['theme'][]).map((t) => (
              <label key={t} className="radio-row">
                <input
                  type="radio"
                  name="theme"
                  value={t}
                  checked={localTheme === t}
                  onChange={() => setLocalTheme(t)}
                />
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </label>
            ))}
          </section>
        </div>

        <div className="modal__footer">
          <button onClick={onClose}>Cancel</button>
          <button className="btn--primary" onClick={apply}>Apply</button>
        </div>
      </div>
    </div>
  );
}
