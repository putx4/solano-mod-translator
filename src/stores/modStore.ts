import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface ModInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  mc_version: string;
  loader: string;
  path: string;
  size_bytes: number;
  lang_files: Array<{ locale: string; keys_count: number; path: string }>;
}

interface ModStore {
  mods: ModInfo[];
  scanning: boolean;
  scanMessage: string;
  scannedFolder: string | null;
  hasScanned: boolean;
  scanFolder: (folder: string) => Promise<void>;
  clearMods: () => void;
}

export const useModStore = create<ModStore>((set) => ({
  mods: [],
  scanning: false,
  scanMessage: "",
  scannedFolder: null,
  hasScanned: false,

  scanFolder: async (folder: string) => {
    set({
      scanning: true,
      scanMessage: `Escaneando ${folder}...`,
      mods: [],
    });

    try {
      const result = await invoke<ModInfo[]>("scan_mods_folder", { folder });
      set({
        mods: result,
        scanning: false,
        scanMessage: "",
        scannedFolder: folder,
        hasScanned: true,
      });
    } catch (error) {
      set({
        scanning: false,
        scanMessage: `❌ Error: ${error}`,
        hasScanned: false,
      });
    }
  },

  clearMods: () => set({ mods: [], hasScanned: false, scannedFolder: null }),
}));