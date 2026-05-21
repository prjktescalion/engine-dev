import { create } from 'zustand';
import type { Component, Entity, TransformComponent } from '../types/engine';

interface SceneState {
  sceneName: string;
  scenePath: string | null;
  entities: Entity[];
  selectedEntityId: string | null;

  setSceneName: (name: string) => void;
  setScenePath: (path: string | null) => void;
  addEntity: (entity: Entity) => void;
  removeEntity: (id: string) => void;
  selectEntity: (id: string | null) => void;
  renameEntity: (id: string, name: string) => void;
  updateComponent: (entityId: string, component: Component) => void;
  updateTransform: (entityId: string, fields: Partial<Omit<TransformComponent, 'type'>>) => void;
  setEntities: (entities: Entity[]) => void;
  clearScene: () => void;
}

export const useSceneStore = create<SceneState>((set) => ({
  sceneName: 'Untitled Scene',
  scenePath: null,
  entities: [],
  selectedEntityId: null,

  setSceneName: (name) => set({ sceneName: name }),
  setScenePath: (path) => set({ scenePath: path }),

  addEntity: (entity) =>
    set((s) => ({ entities: [...s.entities, entity] })),

  removeEntity: (id) =>
    set((s) => ({
      entities: s.entities.filter((e) => e.id !== id),
      selectedEntityId: s.selectedEntityId === id ? null : s.selectedEntityId,
    })),

  selectEntity: (id) => set({ selectedEntityId: id }),

  renameEntity: (id, name) =>
    set((s) => ({
      entities: s.entities.map((e) => (e.id === id ? { ...e, name } : e)),
    })),

  updateComponent: (entityId, component) =>
    set((s) => ({
      entities: s.entities.map((e) => {
        if (e.id !== entityId) return e;
        const idx = e.components.findIndex((c) => c.type === component.type);
        const components =
          idx >= 0
            ? e.components.map((c, i) => (i === idx ? component : c))
            : [...e.components, component];
        return { ...e, components };
      }),
    })),

  updateTransform: (entityId, fields) =>
    set((s) => ({
      entities: s.entities.map((e) => {
        if (e.id !== entityId) return e;
        const components = e.components.map((c) => {
          if (c.type !== 'transform') return c;
          return { ...c, ...fields } as TransformComponent;
        });
        return { ...e, components };
      }),
    })),

  setEntities: (entities) => set({ entities, selectedEntityId: null }),

  clearScene: () =>
    set({ entities: [], selectedEntityId: null, sceneName: 'Untitled Scene', scenePath: null }),
}));
