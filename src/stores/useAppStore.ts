import { create } from "zustand";
import { api, errorMessage } from "@/lib/api";
import { applyTheme, normalizeTheme } from "@/lib/themes";
import type { AppSettings, DownloadProgress, JavaRuntime, SystemInfo } from "@/types";

type AppState = {
  settings: AppSettings | null;
  system: SystemInfo | null;
  javas: JavaRuntime[];
  downloads: DownloadProgress[];
  ready: boolean;
  offline: boolean;
  load: () => Promise<void>;
  saveSettings: (value: AppSettings) => Promise<void>;
};

export const useAppStore = create<AppState>((set) => ({
  settings: null,
  system: null,
  javas: [],
  downloads: [],
  ready: false,
  offline: false,
  load: async () => {
    try {
      const [settings, system, javas, downloads] = await Promise.all([
        api.settings(),
        api.system(),
        api.detectJava().catch(() => []),
        api.downloads().catch(() => []),
      ]);
      set({
        settings,
        system,
        javas,
        downloads,
        offline: system.offline,
        ready: true,
      });
      applyTheme(settings.theme);
      localStorage.setItem("serverforge:theme", normalizeTheme(settings.theme));
    } catch (err) {
      console.error(errorMessage(err));
      set({ ready: true });
    }
  },
  saveSettings: async (value) => {
    await api.saveSettings(value);
    localStorage.setItem("serverforge:theme", normalizeTheme(value.theme));
    applyTheme(value.theme);
    set({ settings: value });
  },
}));
