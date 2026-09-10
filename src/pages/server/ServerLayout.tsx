import { NavLink, Outlet, useLocation, useParams } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { StatusDot } from "@/components/server/StatusDot";
import { PageMotion } from "@/components/layout/PageMotion";
import { supportsMods, supportsPlugins } from "@/lib/providers";
import { useServerStore } from "@/stores/useServerStore";

export function ServerLayout() {
  const { id } = useParams();
  const location = useLocation();
  const server = useServerStore((s) => s.servers.find((x) => x.id === id));
  const loading = useServerStore((s) => s.loading);
  if (!server) {
    if (loading) {
      return <div className="p-8 text-sm text-muted-foreground">Loading server…</div>;
    }
    return <div className="p-8 text-sm text-muted-foreground">Server not found.</div>;
  }

  const tabs = [
    ["", "Overview"],
    ["console", "Console"],
    ["players", "Players"],
    ...(supportsPlugins(server.provider) ? [["plugins", "Plugins"]] : []),
    ...(supportsMods(server.provider) ? [["mods", "Mods"]] : []),
    ["files", "Files"],
    ["worlds", "Worlds"],
    ["backups", "Backups"],
    ["settings", "Settings"],
    ["updates", "Updates"],
  ] as const;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="shrink-0 border-b border-border/70 bg-card/40 px-6 py-4 backdrop-blur-md">
        <NavLink to="/servers" className="mb-2 inline-flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground">
          <ArrowLeft className="size-3" /> Servers
        </NavLink>
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-semibold tracking-tight">{server.name}</h1>
          <StatusDot status={server.status} />
        </div>
        <div className="mt-1 text-sm text-muted-foreground">
          {server.provider} {server.minecraftVersion} · localhost:{server.port}
        </div>
        <div className="mt-4 flex gap-1 overflow-auto">
          {tabs.map(([path, label]) => (
            <NavLink
              key={label}
              to={path ? `/servers/${server.id}/${path}` : `/servers/${server.id}`}
              end={!path}
              className={({ isActive }) =>
                `rounded-full px-3 py-1.5 text-sm transition-all duration-200 ${
                  isActive
                    ? "bg-primary text-primary-foreground shadow-[0_0_20px_oklch(0.78_0.15_55_/_0.28)]"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }`
              }
            >
              {label}
            </NavLink>
          ))}
        </div>
      </div>
      <div className="min-h-0 flex-1 overflow-auto p-6">
        <PageMotion key={location.pathname} className="h-full min-h-0">
          <Outlet />
        </PageMotion>
      </div>
    </div>
  );
}
