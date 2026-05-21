import { useEffect, useRef, useCallback } from 'react';
import { Assets, Graphics, Sprite } from 'pixi.js';
import { invoke } from '@tauri-apps/api/core';
import { usePixiApp } from './usePixiApp';
import { useSceneStore } from '../../store/sceneStore';
import type { Entity, TransformComponent } from '../../types/engine';

const GRID_SIZE = 32;

function makeId() {
  return Math.random().toString(36).slice(2, 10);
}

function drawGrid(app: import('pixi.js').Application) {
  const W = app.screen.width;
  const H = app.screen.height;

  // Remove old grid
  const old = app.stage.children.find((c) => (c as any).__isGrid);
  if (old) app.stage.removeChild(old);

  const g = new Graphics();
  (g as any).__isGrid = true;

  // Background
  g.rect(0, 0, W, H).fill({ color: 0x0a0e14 });

  // Grid lines
  for (let x = 0; x <= W; x += GRID_SIZE) {
    g.moveTo(x, 0).lineTo(x, H);
  }
  for (let y = 0; y <= H; y += GRID_SIZE) {
    g.moveTo(0, y).lineTo(W, y);
  }
  g.stroke({ color: 0x1c2333, width: 1, alpha: 0.6 });

  app.stage.addChildAt(g, 0);
}

export function SceneCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);
  const appRef = usePixiApp(containerRef);
  const spriteMap = useRef<Map<string, Sprite>>(new Map());
  const resizeObRef = useRef<ResizeObserver | null>(null);

  const { entities, selectedEntityId, selectEntity, addEntity, updateTransform } = useSceneStore();

  // Draw grid once app is ready, redraw on container resize
  useEffect(() => {
    const tryInit = setInterval(() => {
      const app = appRef.current;
      if (!app?.renderer) return;
      clearInterval(tryInit);

      drawGrid(app);

      resizeObRef.current = new ResizeObserver(() => {
        if (appRef.current?.renderer) drawGrid(appRef.current);
      });
      if (containerRef.current) resizeObRef.current.observe(containerRef.current);
    }, 50);

    return () => {
      clearInterval(tryInit);
      resizeObRef.current?.disconnect();
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync Zustand entity transforms → PixiJS sprites (and selection highlight)
  useEffect(() => {
    entities.forEach((entity) => {
      const sprite = spriteMap.current.get(entity.id);
      if (!sprite) return;
      const t = entity.components.find((c) => c.type === 'transform') as TransformComponent | undefined;
      if (t) {
        sprite.x = t.x;
        sprite.y = t.y;
        sprite.scale.x = t.scaleX;
        sprite.scale.y = t.scaleY;
        sprite.rotation = t.rotation;
      }
      sprite.tint = selectedEntityId === entity.id ? 0x58a6ff : 0xffffff;
    });
  }, [entities, selectedEntityId]);

  const attachSprite = useCallback(
    (entity: Entity, sprite: Sprite) => {
      sprite.eventMode = 'static';
      sprite.cursor = 'pointer';
      sprite.anchor.set(0.5);

      let dragging = false;
      let dragStart = { x: 0, y: 0, ox: 0, oy: 0 };

      sprite.on('pointerdown', (e) => {
        selectEntity(entity.id);
        dragging = true;
        dragStart = { x: e.global.x, y: e.global.y, ox: sprite.x, oy: sprite.y };
        sprite.zIndex = 100;
        e.stopPropagation();
      });

      sprite.on('pointermove', (e) => {
        if (!dragging) return;
        sprite.x = dragStart.ox + (e.global.x - dragStart.x);
        sprite.y = dragStart.oy + (e.global.y - dragStart.y);
      });

      const endDrag = () => {
        if (!dragging) return;
        dragging = false;
        sprite.zIndex = 1;
        updateTransform(entity.id, { x: sprite.x, y: sprite.y });
      };

      sprite.on('pointerup', endDrag);
      sprite.on('pointerupoutside', endDrag);

      spriteMap.current.set(entity.id, sprite);
      appRef.current?.stage.addChild(sprite);
    },
    [appRef, selectEntity, updateTransform]
  );

  const onDrop = useCallback(
    async (e: React.DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      const raw = e.dataTransfer.getData('application/json');
      if (!raw || !appRef.current) return;

      const { path, kind } = JSON.parse(raw) as { path: string; kind: string; id: string };
      if (kind !== 'image') return;

      const rect = containerRef.current!.getBoundingClientRect();
      const dropX = e.clientX - rect.left;
      const dropY = e.clientY - rect.top;

      try {
        const dataUrl = await invoke<string>('read_image', { path });
        const texture = await Assets.load(dataUrl);
        const sprite = new Sprite(texture);
        sprite.x = dropX;
        sprite.y = dropY;

        const entityId = makeId();
        const entityName = path.split('/').pop()?.replace(/\.[^.]+$/, '') ?? 'Sprite';

        const entity: Entity = {
          id: entityId,
          name: entityName,
          components: [
            { type: 'transform', x: dropX, y: dropY, scaleX: 1, scaleY: 1, rotation: 0 },
            { type: 'sprite', assetPath: path, dataUrl },
          ],
        };

        addEntity(entity);
        attachSprite(entity, sprite);
        selectEntity(entityId);
      } catch (err) {
        console.error('Failed to load image:', err);
      }
    },
    [appRef, containerRef, addEntity, attachSprite, selectEntity]
  );

  return (
    <div
      ref={containerRef}
      className="scene-canvas"
      onDragOver={(e) => e.preventDefault()}
      onDrop={onDrop}
      onClick={() => selectEntity(null)}
    />
  );
}
