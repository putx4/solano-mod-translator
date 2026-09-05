import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface TranslationJob {
  job_id: string;
  mod_id: string;
  mod_name: string;
  total: number;
  translated: number;
  cached: number;
  failed: number;
  status: string;
  current_key: string | null;
  percent: number;
}

interface TranslationStore {
  activeJobs: Record<string, TranslationJob>;
  completedJobs: TranslationJob[];
  isTranslating: boolean;
  unlisten: UnlistenFn | null;

  startTranslation: (
    jarPath: string,
    outputFolder: string,
    modId: string,
    modName: string,
    sourceLocale?: string,
    targetLocale?: string
  ) => Promise<void>;
  setupListeners: () => Promise<void>;
  cleanup: () => void;
}

export const useTranslationStore = create<TranslationStore>((set, get) => ({
  activeJobs: {},
  completedJobs: [],
  isTranslating: false,
  unlisten: null,

  setupListeners: async () => {
    const { unlisten } = get();
    if (unlisten) return; // Ya está escuchando

    const unProgress = await listen<TranslationJob>("translation-progress", (event) => {
      const job = event.payload;
      set((state) => ({
        activeJobs: {
          ...state.activeJobs,
          [job.job_id]: job,
        },
        isTranslating: true,
      }));
    });

    const unComplete = await listen<TranslationJob>("translation-complete", (event) => {
      const job = event.payload;
      set((state) => {
        const { [job.job_id]: removed, ...rest } = state.activeJobs;
        return {
          activeJobs: rest,
          completedJobs: [...state.completedJobs, job],
          isTranslating: Object.keys(rest).length > 0,
        };
      });
    });

    const unError = await listen<TranslationJob>("translation-error", (event) => {
      const job = event.payload;
      set((state) => {
        const { [job.job_id]: removed, ...rest } = state.activeJobs;
        return {
          activeJobs: rest,
          isTranslating: Object.keys(rest).length > 0,
        };
      });
      console.error(`Error en traducción de ${job.mod_name}: ${job.status}`);
    });

    // Guardar TODOS los unlisten para limpiarlos correctamente
    set({
      unlisten: () => {
        unProgress();
        unComplete();
        unError();
      },
    });
  },

  startTranslation: async (jarPath, outputFolder, modId, modName, sourceLocale, targetLocale) => {
    try {
      await get().setupListeners();
      const jobId = await invoke<string>("translate_mod", {
        jarPath,
        outputFolder,
        modId,
        modName,
        sourceLocale: sourceLocale ?? null,
        targetLocale: targetLocale ?? null,
      });
      console.log("Translation started with job ID:", jobId);
    } catch (error) {
      console.error("Error starting translation:", error);
      alert(`Error al iniciar traducción: ${error}`);
    }
  },

  cleanup: () => {
    const { unlisten } = get();
    if (unlisten) {
      unlisten();
      set({ unlisten: null });
    }
  },
}));