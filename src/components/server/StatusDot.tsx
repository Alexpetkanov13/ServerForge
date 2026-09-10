import { cn } from "@/lib/utils";
import type { ServerStatus } from "@/types";

const labels: Record<ServerStatus, string> = {
  online: "Online",
  offline: "Offline",
  starting: "Starting",
  stopping: "Stopping",
  crashed: "Crashed",
  installing: "Installing",
  updating: "Updating",
  error: "Error",
};

const colors: Record<ServerStatus, string> = {
  online: "bg-emerald-400 sf-status-glow",
  offline: "bg-zinc-500",
  starting: "bg-amber-400 animate-pulse",
  stopping: "bg-amber-400 animate-pulse",
  crashed: "bg-red-500",
  installing: "bg-sky-400 animate-pulse",
  updating: "bg-sky-400 animate-pulse",
  error: "bg-red-500",
};

export function StatusDot({ status, className }: { status: string; className?: string }) {
  const key = (status as ServerStatus) in labels ? (status as ServerStatus) : "offline";
  return (
    <span className={cn("inline-flex items-center gap-2 text-sm", className)}>
      <span className={cn("size-2 rounded-full", colors[key])} />
      {labels[key]}
    </span>
  );
}
