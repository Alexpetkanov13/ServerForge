import { useEffect } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "@/components/layout/Sidebar";
import { CommandPalette } from "@/components/layout/CommandPalette";
import { UpdateManager } from "@/components/layout/UpdateManager";
import { PageMotion } from "@/components/layout/PageMotion";
import { shellKey } from "@/lib/motion";
import { applyTheme } from "@/lib/themes";
import { useAppStore } from "@/stores/useAppStore";
import { useConsoleStore } from "@/stores/useConsoleStore";
import { useServerStore } from "@/stores/useServerStore";
import type { DownloadProgress, ServerLogLine } from "@/types";

export function AppShell() {
  const theme = useAppStore((s) => s.settings?.theme);
  const fontScale = useAppStore((s) => s.settings?.fontScale);
  const reducedMotion = useAppStore((s) => s.settings?.reducedMotion);
  const refresh = useServerStore((s) => s.refresh);
  const listenServers = useServerStore((s) => s.listen);
  const pushLog = useConsoleStore((s) => s.push);
  const location = useLocation();

  useEffect(() => {
    void refresh();
    let unsub: (() => void) | undefined;
    void listenServers().then((fn) => {
      unsub = fn;
    });
    const unsubs: Array<() => void> = [];
    void listen<ServerLogLine>("server-log", (e) => pushLog(e.payload)).then((u) => unsubs.push(u));
    void listen<DownloadProgress>("download-progress", (e) => {
      useAppStore.setState((s) => {
        const rest = s.downloads.filter((d) => d.id !== e.payload.id);
        return { downloads: [e.payload, ...rest] };
      });
    }).then((u) => unsubs.push(u));
    return () => {
      unsub?.();
      unsubs.forEach((u) => u());
    };
  }, [refresh, listenServers, pushLog]);

  useEffect(() => {
    // applyTheme sets the dark class, data-theme attr, color-scheme AND
    // bulletproof inline variables so every theme visibly applies.
    applyTheme(theme);
    document.documentElement.classList.toggle("reduce-motion", Boolean(reducedMotion));
    document.documentElement.style.fontSize = `${(fontScale ?? 1) * 100}%`;
  }, [theme, fontScale, reducedMotion]);

  return (
    <div className="relative flex h-full overflow-hidden bg-background">
      <div className="pointer-events-none absolute inset-0 sf-ember-field opacity-90" />
      <Sidebar />
      <main className="relative min-h-0 min-w-0 flex-1">
        <div className="absolute inset-0 overflow-auto">
          <PageMotion key={shellKey(location.pathname)} className="h-full min-h-full">
            <Outlet />
          </PageMotion>
        </div>
      </main>
      <CommandPalette />
      <UpdateManager />
    </div>
  );
}
