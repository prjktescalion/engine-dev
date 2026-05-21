import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { useSettingsStore } from '../../store/settingsStore';
import type { Asset } from '../../types/engine';

const ICONS: Record<string, string> = {
  image: '🖼',
  script: '📄',
  audio: '🔊',
  other: '📎',
  dir: '📁',
};

const LANG_EXT: Record<string, 'rust' | 'java' | 'python'> = {
  rs: 'rust',
  java: 'java',
  py: 'python',
};

interface AssetItemProps {
  asset: Asset;
  depth?: number;
}

export function AssetItem({ asset, depth = 0 }: AssetItemProps) {
  const [expanded, setExpanded] = useState(false);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const { editor, custom_path } = useSettingsStore();

  const ext = asset.name.split('.').pop()?.toLowerCase() ?? '';
  const isScript = asset.kind === 'script';

  function onDragStart(e: React.DragEvent) {
    e.dataTransfer.setData(
      'application/json',
      JSON.stringify({ id: asset.id, path: asset.path, kind: asset.kind })
    );
    e.dataTransfer.effectAllowed = 'copy';
  }

  async function openInEditor() {
    try {
      await invoke('open_in_editor', {
        filePath: asset.path,
        editorOverride: editor === 'custom' ? custom_path : null,
      });
    } catch (e) {
      alert(`Failed to open editor: ${e}`);
    }
    setContextMenu(null);
  }

  function onContextMenu(e: React.MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (isScript) setContextMenu({ x: e.clientX, y: e.clientY });
  }

  if (asset.is_dir) {
    return (
      <li className="asset-item asset-item--dir" style={{ paddingLeft: `${depth * 12}px` }}>
        <button className="asset-item__row" onClick={() => setExpanded((p) => !p)}>
          <span className="asset-item__chevron">{expanded ? '▾' : '▸'}</span>
          <span className="asset-item__icon">📁</span>
          <span className="asset-item__name">{asset.name}</span>
        </button>
        {expanded && asset.children.length > 0 && (
          <ul className="asset-list">
            {asset.children.map((child) => (
              <AssetItem key={child.id} asset={child} depth={depth + 1} />
            ))}
          </ul>
        )}
      </li>
    );
  }

  return (
    <>
      <li
        className={`asset-item asset-item--${asset.kind}`}
        style={{ paddingLeft: `${depth * 12 + 16}px` }}
        draggable={asset.kind === 'image'}
        onDragStart={asset.kind === 'image' ? onDragStart : undefined}
        onContextMenu={onContextMenu}
        onDoubleClick={isScript ? openInEditor : undefined}
        title={asset.path}
      >
        <span className="asset-item__icon">{ICONS[asset.kind] ?? '📎'}</span>
        <span className="asset-item__name">{asset.name}</span>
        {isScript && (
          <span className={`script-lang script-lang--${LANG_EXT[ext] ?? 'other'}`}>
            {LANG_EXT[ext] ?? ext}
          </span>
        )}
      </li>

      {contextMenu && (
        <ul
          className="context-menu"
          style={{ top: contextMenu.y, left: contextMenu.x }}
          onClick={() => setContextMenu(null)}
        >
          <li>
            <button onClick={openInEditor}>Open in Editor</button>
          </li>
        </ul>
      )}
    </>
  );
}
