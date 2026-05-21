import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as dialogOpen, save as dialogSave } from '@tauri-apps/plugin-dialog';
import { useSceneStore } from '../../store/sceneStore';
import { useAssetStore } from '../../store/assetStore';
import type { Entity } from '../../types/engine';

interface MenuBarProps {
  onOpenSettings: () => void;
}

export function MenuBar({ onOpenSettings }: MenuBarProps) {
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const { sceneName, scenePath, entities, setScenePath, setSceneName, setEntities, clearScene } =
    useSceneStore();
  const { setProjectRoot } = useAssetStore();

  const toggle = (menu: string) => setOpenMenu((p) => (p === menu ? null : menu));
  const close = () => setOpenMenu(null);

  async function openProject() {
    close();
    const folder = await dialogOpen({ directory: true, multiple: false, title: 'Open Project Folder' });
    if (folder) setProjectRoot(folder as string);
  }

  async function saveScene() {
    close();
    let path = scenePath;
    if (!path) {
      path = await dialogSave({ title: 'Save Scene', filters: [{ name: 'Scene', extensions: ['ndscene'] }] });
      if (!path) return;
      setScenePath(path as string);
    }
    const data = JSON.stringify({ version: 1, name: sceneName, entities }, null, 2);
    await invoke('save_scene', { path, data });
  }

  async function loadScene() {
    close();
    const file = await dialogOpen({ multiple: false, filters: [{ name: 'Scene', extensions: ['ndscene'] }] });
    if (!file) return;
    const raw = await invoke<string>('load_scene', { path: file as string });
    const parsed = JSON.parse(raw) as { version: number; name: string; entities: Entity[] };
    setSceneName(parsed.name);
    setScenePath(file as string);
    setEntities(parsed.entities);
  }

  const menus: { label: string; items: { label: string; action: () => void; divider?: boolean }[] }[] = [
    {
      label: 'File',
      items: [
        { label: 'Open Project...', action: openProject },
        { label: 'Load Scene...', action: loadScene },
        { label: 'Save Scene', action: saveScene },
        { label: 'New Scene', action: () => { close(); clearScene(); }, divider: true },
      ],
    },
    {
      label: 'Project',
      items: [
        { label: 'Settings...', action: () => { close(); onOpenSettings(); } },
      ],
    },
  ];

  return (
    <header className="menu-bar" onClick={(e) => e.stopPropagation()}>
      <span className="menu-bar__logo">NeuDel-II</span>
      <nav className="menu-bar__nav">
        {menus.map((menu) => (
          <div key={menu.label} className="menu-bar__item">
            <button
              className={`menu-bar__trigger${openMenu === menu.label ? ' active' : ''}`}
              onClick={() => toggle(menu.label)}
            >
              {menu.label}
            </button>
            {openMenu === menu.label && (
              <ul className="menu-bar__dropdown">
                {menu.items.map((item) => (
                  <li key={item.label} className={item.divider ? 'has-divider' : ''}>
                    <button onClick={item.action}>{item.label}</button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        ))}
      </nav>
      <span className="menu-bar__scene-name">{sceneName}</span>
    </header>
  );
}
