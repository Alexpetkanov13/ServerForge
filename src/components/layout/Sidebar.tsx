import { NavLink, useLocation } from "react-router-dom";
import { motion } from "framer-motion";
import {
  Anvil,
  Download,
  LayoutDashboard,
  Plus,
  Server,
  Settings,
  Sparkles,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useServerStore } from "@/stores/useServerStore";
import logo from "@/assets/logo.webp";

const items = [
  { to: "/", label: "Dashboard", icon: LayoutDashboard },
  { to: "/servers", label: "Servers", icon: Server },
  { to: "/create", label: "Create Server", icon: Plus },
  { to: "/templates", label: "Templates", icon: Sparkles },
  { to: "/downloads", label: "Downloads", icon: Download },
  { to: "/java", label: "Java", icon: Anvil },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  const servers = useServerStore((s) => s.servers);
  const location = useLocation();
  const online = servers.filter((s) => s.status === "online" || s.status === "starting").length;

  return (
    <aside className="relative z-10 flex h-full w-64 shrink-0 flex-col border-r border-sidebar-border bg-sidebar/80 px-3 py-4 backdrop-blur-xl">
      <div className="mb-6 flex items-center gap-3 px-2">
        <img
          src={logo}
          alt="ServerForge"
          width={40}
          height={40}
          className="size-10 rounded-xl shadow-[0_0_24px_oklch(0.78_0.15_55_/_0.35)]"
        />
        <div>
          <div className="text-sm font-semibold tracking-tight">ServerForge</div>
          <div className="text-[11px] text-muted-foreground">Forge. Launch. Command.</div>
        </div>
      </div>
      <nav className="flex flex-1 flex-col gap-0.5">
        {items.map((item) => {
          const Icon = item.icon;
          const active =
            item.to === "/"
              ? location.pathname === "/"
              : location.pathname.startsWith(item.to);
          return (
            <NavLink
              key={item.to}
              to={item.to}
              className={cn(
                "relative flex items-center gap-2 rounded-xl px-2.5 py-2 text-sm transition-colors",
                active ? "text-foreground" : "text-muted-foreground hover:text-foreground",
              )}
            >
              {active ? (
                <motion.span
                  layoutId="nav-active"
                  className="absolute inset-0 rounded-xl bg-sidebar-accent shadow-[inset_0_0_0_1px_oklch(0.78_0.15_55_/_0.28)]"
                  transition={{ type: "spring", stiffness: 380, damping: 32 }}
                />
              ) : null}
              <Icon className="relative size-4" />
              <span className="relative">{item.label}</span>
              {item.to === "/servers" && online > 0 ? (
                <span className="relative ml-auto rounded-full bg-emerald-500/15 px-1.5 text-[10px] text-emerald-400">
                  {online}
                </span>
              ) : null}
            </NavLink>
          );
        })}
      </nav>
      <div className="rounded-xl border border-border/70 bg-background/40 p-3 text-xs text-muted-foreground">
        {servers.length} server{servers.length === 1 ? "" : "s"} · press
        <kbd className="mx-1 rounded bg-muted px-1 py-0.5 font-mono text-[10px]">Ctrl K</kbd>
      </div>
    </aside>
  );
}
