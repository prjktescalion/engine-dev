import { useState } from 'react';
import { MenuBar } from '../menubar/MenuBar';
import { HierarchyPanel } from '../hierarchy/HierarchyPanel';
import { SceneCanvas } from '../canvas/SceneCanvas';
import { InspectorPanel } from '../inspector/InspectorPanel';
import { AssetBrowser } from '../assets/AssetBrowser';
import { EditorSettingsModal } from '../settings/EditorSettings';

export function AppShell() {
  const [showSettings, setShowSettings] = useState(false);
  const [bottomTab, setBottomTab] = useState<'assets' | 'console'>('assets');

  return (
    <div className="app-shell">
      <MenuBar onOpenSettings={() => setShowSettings(true)} />

      <div className="app-shell__body">
        <HierarchyPanel />

        <main className="app-shell__center">
          <SceneCanvas />
          <div className="bottom-panel">
            <div className="bottom-panel__tabs">
              <button
                className={bottomTab === 'assets' ? 'active' : ''}
                onClick={() => setBottomTab('assets')}
              >
                Assets
              </button>
              <button
                className={bottomTab === 'console' ? 'active' : ''}
                onClick={() => setBottomTab('console')}
              >
                Console
              </button>
            </div>
            <div className="bottom-panel__content">
              {bottomTab === 'assets' && <AssetBrowser />}
              {bottomTab === 'console' && (
                <div className="console-panel">
                  <span className="muted">NeuDel-II Studio ready.</span>
                </div>
              )}
            </div>
          </div>
        </main>

        <InspectorPanel />
      </div>

      {showSettings && <EditorSettingsModal onClose={() => setShowSettings(false)} />}
    </div>
  );
}
