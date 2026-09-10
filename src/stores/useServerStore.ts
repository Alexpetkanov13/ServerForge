import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { api } from "@/lib/api";
import type { CrashInfo, ServerMetrics, ServerRecord } from "@/types";

type ServerState = {
  servers: ServerRecord[];
  metrics: Record<string, ServerMetrics>;
  crashes: Record<string, CrashInfo>;
  loading: boolean;
  refresh: () => Promise<void>;
  listen: () => Promise<() => void>;
};

export const useServerStore = create<ServerState>((set, get) => ({
  servers: [],
  metrics: {},
  crashes: {},
  loading: false,
  refresh: async () => {
    set({ loading: true });
    try {
      const servers = await api.servers();
      set({ servers, loading: false });
    } catch {
      set({ loading: false });
    }
  },
  listen: async () => {
    const unsubs = await Promise.all([
      listen<ServerMetrics>("server-metrics", (event) => {
        set((s) => ({ metrics: { ...s.metrics, [event.payload.serverId]: event.payload } }));
      }),
      listen<{ serverId: string; status: string }>("server-status", (event) => {
        // Drop stale live metrics once the process is gone so Overview never
        // shows frozen player counts for an offline server.
        if (event.payload.status === "offline" || event.payload.status === "crashed") {
          set((s) => {
            if (!(event.payload.serverId in s.metrics)) return s;
            const metrics = { ...s.metrics };
            delete metrics[event.payload.serverId];
            return { metrics };
          });
        }
        void get().refresh();
      }),
      listen<CrashInfo>("server-crash", (event) => {
        set((s) => ({ crashes: { ...s.crashes, [event.payload.serverId]: event.payload } }));
        void get().refresh();
      }),
    ]);
    return () => unsubs.forEach((u) => u());
  },
}));
