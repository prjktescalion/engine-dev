import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { EditorSettings } from '../types/engine';

interface SettingsState extends EditorSettings {
  loaded: boolean;
  load: () => Promise<void>;
  save: (patch: Partial<EditorSettings>) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  editor: 'vscode',
  custom_path: '',
  theme: 'dark',
  loaded: false,

  load: async () => {
    try {
      const s = await invoke<EditorSettings>('get_settings');
      set({ ...s, loaded: true });
    } catch {
      set({ loaded: true });
    }
  },

  save: async (patch) => {
    const { editor, custom_path, theme } = { ...get(), ...patch };
    const settings: EditorSettings = { editor, custom_path, theme };
    set(settings);
    try {
      await invoke('save_settings', { settings });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  },
}));
