import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Asset } from '../types/engine';

interface AssetState {
  projectRoot: string | null;
  assets: Asset[];
  loading: boolean;
  error: string | null;

  setProjectRoot: (root: string) => void;
  refreshAssets: () => Promise<void>;
}

export const useAssetStore = create<AssetState>((set, get) => ({
  projectRoot: null,
  assets: [],
  loading: false,
  error: null,

  setProjectRoot: (root) => {
    set({ projectRoot: root });
    get().refreshAssets();
  },

  refreshAssets: async () => {
    const root = get().projectRoot;
    if (!root) return;
    set({ loading: true, error: null });
    try {
      const assets = await invoke<Asset[]>('list_dir', { path: root });
      set({ assets, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },
}));
