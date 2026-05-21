import { useState } from 'react';
import { useSceneStore } from '../../store/sceneStore';

export function HierarchyPanel() {
  const { entities, selectedEntityId, selectEntity, removeEntity, renameEntity } =
    useSceneStore();
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; id: string } | null>(null);

  function startRename(id: string, name: string) {
    setRenamingId(id);
    setRenameValue(name);
    setContextMenu(null);
  }

  function commitRename(id: string) {
    if (renameValue.trim()) renameEntity(id, renameValue.trim());
    setRenamingId(null);
  }

  function onContextMenu(e: React.MouseEvent, id: string) {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, id });
  }

  return (
    <aside
      className="panel hierarchy-panel"
      onClick={() => setContextMenu(null)}
    >
      <div className="panel__header">Hierarchy</div>
      <ul className="hierarchy-list">
        {entities.length === 0 && (
          <li className="hierarchy-list__empty">No entities — drop assets onto canvas</li>
        )}
        {entities.map((entity) => (
          <li
            key={entity.id}
            className={`hierarchy-list__item${selectedEntityId === entity.id ? ' selected' : ''}`}
            onClick={() => selectEntity(entity.id)}
            onContextMenu={(e) => onContextMenu(e, entity.id)}
          >
            <span className="hierarchy-list__icon">◈</span>
            {renamingId === entity.id ? (
              <input
                autoFocus
                className="hierarchy-list__rename"
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                onBlur={() => commitRename(entity.id)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') commitRename(entity.id);
                  if (e.key === 'Escape') setRenamingId(null);
                }}
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <span className="hierarchy-list__name">{entity.name}</span>
            )}
          </li>
        ))}
      </ul>

      {contextMenu && (
        <ul
          className="context-menu"
          style={{ top: contextMenu.y, left: contextMenu.x }}
          onClick={(e) => e.stopPropagation()}
        >
          <li>
            <button
              onClick={() => {
                const e = entities.find((e) => e.id === contextMenu.id);
                if (e) startRename(e.id, e.name);
              }}
            >
              Rename
            </button>
          </li>
          <li>
            <button
              className="danger"
              onClick={() => {
                removeEntity(contextMenu.id);
                setContextMenu(null);
              }}
            >
              Delete
            </button>
          </li>
        </ul>
      )}
    </aside>
  );
}
