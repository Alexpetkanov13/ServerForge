import { create } from "zustand";
import { toast } from "sonner";
import type { Update } from "@tauri-apps/plugin-updater";
import { checkForUpdates, downloadOnly, installDownloaded, metaOf, type UpdateMeta } from "@/lib/updates";
import { useAppStore } from "@/stores/useAppStore";

export type UpdatePhase = "idle" | "checking" | "available" | "downloading" | "installing" | "error";

type UpdateState = {
  phase: UpdatePhase;
  meta: UpdateMeta | null;
  downloaded: number;
  total: number | null;
  error: string | null;
  checkedThisSession: boolean;
  /** Hides the dialog without cancelling an in-flight download/install. */
  dismissed: boolean;
  /** Silent (launch) or manual (settings button) check. Manual bypasses skip. */
  checkNow: (manual?: boolean) => Promise<void>;
  startInstall: () => Promise<void>;
  dismiss: () => void;
  skipVersion: () => Promise<void>;
  reset: () => void;
};

// The live plugin resource lives outside React state (non-serializable).
let liveUpdate: Update | null = null;

async function closeLive() {
  try {
    await liveUpdate?.close();
  } catch {
    // Already consumed after install — harmless.
  }
  liveUpdate = null;
}

export const useUpdateStore = create<UpdateState>((set, get) => ({
  phase: "idle",
  meta: null,
  downloaded: 0,
  total: null,
  error: null,
  checkedThisSession: false,
  dismissed: false,

  checkNow: async (manual = false) => {
    const { phase } = get();
    if (phase === "checking" || phase === "downloading" || phase === "installing") return;
    set({ phase: "checking", error: null });
    const settings = useAppStore.getState().settings;
    const { update, error } = await checkForUpdates(manual);
    if (!update) {
      set({ phase: "idle", checkedThisSession: true, error: error ?? null });
      if (manual) {
        if (error) toast.error(`Update check failed: ${error}`);
        else toast.success("You're on the latest version");
      }
      return;
    }
    const meta = metaOf(update);
    // Respect "skip this version" for automatic checks only.
    if (!manual && settings?.skippedAppVersion && settings.skippedAppVersion === meta.version) {
      await closeLive();
      set({ phase: "idle", checkedThisSession: true, meta: null });
      return;
    }
    await closeLive();
    liveUpdate = update;
    set({ phase: "available", meta, checkedThisSession: true, downloaded: 0, total: null, dismissed: false });

    // Fully automatic mode: download + install without asking.
    if (!manual && settings?.appAutoInstall) {
      await get().startInstall();
    }
  },

  /** Download (with progress), then install. Called for auto mode and for the dialog's confirm button. */
  startInstall: async () => {
    const update = liveUpdate;
    if (!update) return;
    try {
      set({ phase: "downloading", downloaded: 0, total: null, error: null });
      await downloadOnly(update, (d, t) => set({ downloaded: d, total: t }));
      set({ phase: "installing" });
      // On Windows the installer takes over here and the app exits.
      await installDownloaded(update);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      // Re-show the dialog with the error even if it was hidden mid-download.
      set({ phase: "error", error: message, dismissed: false });
      toast.error(`Update failed: ${message}`);
    }
  },

  dismiss: () => {
    const { phase } = get();
    // During download/install only hide — the work continues in background.
    if (phase === "downloading" || phase === "installing") {
      set({ dismissed: true });
    } else {
      set({ phase: "idle", error: null, dismissed: false });
    }
  },

  skipVersion: async () => {
    const { meta } = get();
    const settings = useAppStore.getState().settings;
    if (settings && meta) {
      try {
        await useAppStore.getState().saveSettings({ ...settings, skippedAppVersion: meta.version });
        toast.success(`Skipped version ${meta.version}`);
      } catch {
        // Persist failure shouldn't block dismissal.
      }
    }
    await closeLive();
    set({ phase: "idle", meta: null });
  },

  reset: () => {
    void closeLive();
    set({ phase: "idle", meta: null, downloaded: 0, total: null, error: null, dismissed: false });
  },
}));
