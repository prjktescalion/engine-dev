import { invoke } from '@tauri-apps/api/core';
import { useSceneStore } from '../../store/sceneStore';
import { useSettingsStore } from '../../store/settingsStore';
import type { TransformComponent, SpriteComponent, ScriptComponent } from '../../types/engine';

export function InspectorPanel() {
  const { entities, selectedEntityId, updateTransform } = useSceneStore();
  const { editor, custom_path } = useSettingsStore();

  const entity = entities.find((e) => e.id === selectedEntityId);
  const transform = entity?.components.find((c) => c.type === 'transform') as
    | TransformComponent
    | undefined;
  const sprite = entity?.components.find((c) => c.type === 'sprite') as SpriteComponent | undefined;
  const scripts = entity?.components.filter((c) => c.type === 'script') as ScriptComponent[];

  function numInput(
    label: string,
    value: number,
    onChange: (v: number) => void,
    step = 1
  ) {
    return (
      <div className="inspector__field">
        <label>{label}</label>
        <input
          type="number"
          step={step}
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
        />
      </div>
    );
  }

  async function openScript(filePath: string) {
    try {
      await invoke('open_in_editor', {
        filePath,
        editorOverride: editor === 'custom' ? custom_path : null,
      });
    } catch (e) {
      alert(`Failed to open editor: ${e}`);
    }
  }

  if (!entity) {
    return (
      <aside className="panel inspector-panel">
        <div className="panel__header">Inspector</div>
        <p className="inspector-panel__empty">Select an entity to inspect</p>
      </aside>
    );
  }

  return (
    <aside className="panel inspector-panel">
      <div className="panel__header">Inspector — {entity.name}</div>

      {transform && (
        <section className="inspector__section">
          <div className="inspector__section-title">Transform</div>
          <div className="inspector__row">
            {numInput('X', transform.x, (v) =>
              updateTransform(entity.id, { x: v })
            )}
            {numInput('Y', transform.y, (v) =>
              updateTransform(entity.id, { y: v })
            )}
          </div>
          <div className="inspector__row">
            {numInput('Scale X', transform.scaleX, (v) =>
              updateTransform(entity.id, { scaleX: v }), 0.05
            )}
            {numInput('Scale Y', transform.scaleY, (v) =>
              updateTransform(entity.id, { scaleY: v }), 0.05
            )}
          </div>
          {numInput('Rotation (rad)', transform.rotation, (v) =>
            updateTransform(entity.id, { rotation: v }), 0.01
          )}
        </section>
      )}

      {sprite && (
        <section className="inspector__section">
          <div className="inspector__section-title">Sprite</div>
          <div className="inspector__field">
            <label>Asset</label>
            <span className="inspector__path" title={sprite.assetPath}>
              {sprite.assetPath.split('/').pop()}
            </span>
          </div>
        </section>
      )}

      {scripts && scripts.length > 0 && (
        <section className="inspector__section">
          <div className="inspector__section-title">Scripts</div>
          {scripts.map((sc, i) => (
            <div key={i} className="inspector__script">
              <span className={`script-lang script-lang--${sc.lang}`}>{sc.lang}</span>
              <span className="inspector__path" title={sc.filePath}>
                {sc.filePath.split('/').pop()}
              </span>
              <button
                className="inspector__open-btn"
                onClick={() => openScript(sc.filePath)}
                title="Open in editor"
              >
                ↗
              </button>
            </div>
          ))}
        </section>
      )}
    </aside>
  );
}
