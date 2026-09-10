import { memo, useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { toast } from "sonner";
import {
  Activity,
  Check,
  Copy,
  Cpu,
  FolderOpen,
  Gauge,
  MemoryStick,
  Play,
  RotateCcw,
  Share2,
  Skull,
  Square,
  Timer,
  Users,
  Wifi,
} from "lucide-react";
import { Area, AreaChart, ResponsiveContainer } from "recharts";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DeleteServerButton } from "@/components/server/DeleteServerButton";
import { StatusDot } from "@/components/server/StatusDot";
import { api, errorMessage } from "@/lib/api";
import { formatBytes, formatDuration } from "@/lib/format";
import { useServerStore } from "@/stores/useServerStore";
import type { SharePackInfo } from "@/types";
import { cn } from "@/lib/utils";

const cardIn = {
  hidden: { opacity: 0, y: 14, scale: 0.99 },
  show: (i: number) => ({
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { duration: 0.34, delay: Math.min(i * 0.05, 0.35), ease: [0.22, 1, 0.36, 1] as const },
  }),
};

const Metric = memo(function Metric({
  index,
  label,
  value,
  icon,
  sub,
  bar,
  pulse,
}: {
  index: number;
  label: string;
  value: string;
  icon: React.ReactNode;
  sub?: string;
  bar?: number;
  pulse?: boolean;
}) {
  return (
    <motion.div variants={cardIn} initial="hidden" animate="show" custom={index}>
      <Card className="sf-card-hover group relative overflow-hidden bg-card/80">
        <span className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary/60 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100" />
        <CardContent className="p-4">
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <span className="text-primary/80">{icon}</span>
            {label}
            {pulse && <i className="sf-blink ml-auto size-1.5 rounded-full bg-emerald-400" />}
          </div>
          <div className="mt-1 truncate text-lg font-semibold tracking-tight">{value}</div>
          {typeof bar === "number" && (
            <div className="mt-2 h-1 overflow-hidden rounded-full bg-muted">
              <motion.div
                className={cn("h-full rounded-full", bar > 0.9 ? "bg-red-400" : "bg-gradient-to-r from-primary to-emerald-400")}
                animate={{ width: `${Math.min(100, Math.max(0, bar * 100))}%` }}
                transition={{ type: "spring", stiffness: 120, damping: 22 }}
              />
            </div>
          )}
          {sub && <div className="mt-1 truncate text-[11px] text-muted-foreground">{sub}</div>}
        </CardContent>
      </Card>
    </motion.div>
  );
});

export function OverviewPage() {
  const { id } = useParams();
  const reduce = useReducedMotion();
  const server = useServerStore((s) => s.servers.find((x) => x.id === id));
  const metrics = useServerStore((s) => (id ? s.metrics[id] : undefined));
  const crash = useServerStore((s) => (id ? s.crashes[id] : undefined));
  const refresh = useServerStore((s) => s.refresh);
  const [sharePack, setSharePack] = useState<SharePackInfo | null>(null);
  const [shareError, setShareError] = useState("");
  const [propsMax, setPropsMax] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!id) return;
    setShareError("");
    void api
      .sharePack(id)
      .then(setSharePack)
      .catch((err) => {
        setSharePack(null);
        setShareError(errorMessage(err));
      });
  }, [id, server?.updatedAt, server?.status]);

  // Configured cap from server.properties — the source of truth for the
  // denominator when live metrics are absent (server offline) or stale.
  useEffect(() => {
    if (!id) return;
    void api
      .properties(id)
      .then((p) => {
        const raw = p.fields.find((f) => f.key === "max-players")?.value;
        const n = raw ? Number.parseInt(raw, 10) : NaN;
        setPropsMax(Number.isFinite(n) && n > 0 ? n : null);
      })
      .catch(() => setPropsMax(null));
  }, [id, server?.updatedAt]);

  const chart = useMemo(
    () =>
      Array.from({ length: 24 }, (_, i) => ({
        i,
        cpu: metrics ? Math.max(0, metrics.cpuPercent - (23 - i) * 0.35) : 0,
      })),
    [metrics?.cpuPercent],
  );

  if (!server || !id) return null;

  const run = async (fn: () => Promise<void>, ok?: string) => {
    setBusy(true);
    try {
      await fn();
      await refresh();
      if (ok) toast.success(ok);
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setBusy(false);
    }
  };

  const copyAddress = () => {
    const addr = `localhost:${server.port}`;
    void navigator.clipboard
      .writeText(addr)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      })
      .catch(() => toast.error("Copy failed"));
  };

  const online = server.status === "online" || server.status === "starting";
  // Live metrics win; configured cap second; 20 only as a last resort.
  const maxPlayers = metrics?.maxPlayers || propsMax || 20;
  const playerCount = metrics?.playerCount ?? 0;
  const memMax = (server.memoryMaxMb || 0) * 1024 * 1024;
  const memRatio = memMax > 0 ? (metrics?.memoryBytes ?? 0) / memMax : 0;

  return (
    <motion.div
      initial={reduce ? false : { opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
      className="space-y-4"
    >
      <AnimatePresence>
        {crash ? (
          <motion.div initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0 }}>
            <Card className="border-destructive/40 bg-destructive/[0.05]">
              <CardHeader>
                <CardTitle>Server crashed</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2 text-sm">
                <p>Reason: {crash.reason}</p>
                <p className="text-muted-foreground">{crash.suggestion}</p>
                <Button onClick={() => void run(() => api.updateMemory(id, server.memoryMinMb, Math.min(server.memoryMaxMb + 2048, 32768)))}>
                  Increase RAM
                </Button>
              </CardContent>
            </Card>
          </motion.div>
        ) : null}
      </AnimatePresence>

      {/* Status hero */}
      <motion.div variants={cardIn} initial="hidden" animate="show" custom={0}>
        <Card className="relative overflow-hidden">
          <div aria-hidden className="pointer-events-none absolute inset-0">
            <motion.span
              animate={reduce || !online ? undefined : { x: ["-20%", "20%", "-20%"] }}
              transition={{ duration: 12, repeat: Infinity, ease: "easeInOut" }}
              className="absolute -top-16 left-1/3 size-64 rounded-full bg-primary/[0.1] blur-3xl"
            />
          </div>
          <CardContent className="relative flex flex-wrap items-center gap-3 p-4">
            <StatusDot status={server.status} />
            <div>
              <div className="flex items-center gap-2 font-semibold">
                {server.status === "online" ? "Online and ready" : server.status === "starting" ? "Warming up…" : server.status === "stopping" ? "Shutting down…" : "Offline"}
              </div>
              <div className="text-xs text-muted-foreground">
                {server.provider} {server.minecraftVersion} · uptime {formatDuration(metrics?.uptimeSeconds ?? 0)}
              </div>
            </div>
            <button
              onClick={copyAddress}
              className="ml-auto flex items-center gap-1.5 rounded-lg border border-border bg-muted/50 px-2.5 py-1.5 font-mono text-xs transition-all hover:-translate-y-0.5 hover:border-primary/50"
              title="Copy address"
            >
              localhost:{server.port}
              {copied ? <Check className="size-3.5 text-emerald-400" /> : <Copy className="size-3.5 text-muted-foreground" />}
            </button>
          </CardContent>
        </Card>
      </motion.div>

      <div className="flex flex-wrap gap-2">
        <motion.div whileTap={reduce ? undefined : { scale: 0.96 }}>
          <Button disabled={busy} onClick={() => void run(() => api.start(id), "Server started")} className="shadow-[0_0_20px_oklch(0.72_0.18_150_/_0.3)]">
            <Play className="size-4" /> Start
          </Button>
        </motion.div>
        <Button variant="outline" disabled={busy} onClick={() => void run(() => api.stop(id), "Server stopped")}>
          <Square className="size-4" /> Stop
        </Button>
        <Button variant="outline" disabled={busy} onClick={() => void run(() => api.restart(id), "Server restarted")}>
          <RotateCcw className="size-4" /> Restart
        </Button>
        <Button variant="destructive" disabled={busy} onClick={() => void run(() => api.stop(id, true), "Server force-stopped")}>
          <Skull className="size-4" /> Force stop
        </Button>
        <DeleteServerButton id={id} name={server.name} variant="outline" />
      </div>

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Metric index={1} label="Players" value={`${playerCount} / ${maxPlayers}`} icon={<Users className="size-3.5" />} bar={maxPlayers > 0 ? playerCount / maxPlayers : 0} pulse={online} sub={propsMax ? `Cap set to ${propsMax} in settings` : undefined} />
        <Metric index={2} label="CPU" value={`${(metrics?.cpuPercent ?? 0).toFixed(0)}%`} icon={<Cpu className="size-3.5" />} bar={(metrics?.cpuPercent ?? 0) / 100} />
        <Metric index={3} label="RAM" value={`${formatBytes(metrics?.memoryBytes ?? 0)} / ${formatBytes(memMax)}`} icon={<MemoryStick className="size-3.5" />} bar={memRatio} />
        <Metric index={4} label="TPS" value={(metrics?.tps ?? 20).toFixed(1)} icon={<Gauge className="size-3.5" />} bar={(metrics?.tps ?? 20) / 20} />
        <Metric index={5} label="Uptime" value={formatDuration(metrics?.uptimeSeconds ?? 0)} icon={<Timer className="size-3.5" />} />
        <Metric index={6} label="Version" value={`${server.provider} ${server.minecraftVersion}`} icon={<Activity className="size-3.5" />} sub={server.build ? `build ${server.build}` : undefined} />
        <Metric index={7} label="Address" value={`localhost:${server.port}`} icon={<Wifi className="size-3.5" />} />
        <Metric index={8} label="Status" value={server.status.toUpperCase()} icon={<span className="inline-block size-2 rounded-full bg-current" />} pulse={online} />
      </div>

      <motion.div variants={cardIn} initial="hidden" animate="show" custom={9}>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0">
            <CardTitle className="flex items-center gap-2 text-sm">
              <Share2 className="size-4" />
              Share this server
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <p className="text-muted-foreground">
              ServerForge keeps a <span className="font-medium text-foreground">{server.name.toLowerCase().replace(/\s+/g, "-")}.server</span> file inside this server folder. It updates automatically when you change the server. Send that file to anyone with ServerForge.
            </p>
            {sharePack ? (
              <div className="flex flex-wrap items-center gap-3 text-muted-foreground">
                <span className="truncate font-mono text-xs">{sharePack.path}</span>
                <span>{formatBytes(sharePack.sizeBytes)}</span>
              </div>
            ) : (
              <p className="text-muted-foreground">{shareError || "The share file will appear here once it is ready."}</p>
            )}
            <Button
              variant="outline"
              disabled={!sharePack}
              className="transition-all hover:-translate-y-0.5"
              onClick={() => {
                if (!sharePack) return;
                void api.openPath(sharePack.path).catch((e) => toast.error(errorMessage(e)));
              }}
            >
              <FolderOpen className="size-4" />
              Open share file
            </Button>
          </CardContent>
        </Card>
      </motion.div>

      <motion.div variants={cardIn} initial="hidden" animate="show" custom={10}>
        <Card className="overflow-hidden">
          <CardHeader>
            <CardTitle className="text-sm">CPU — last samples</CardTitle>
          </CardHeader>
          <CardContent className="h-40">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chart}>
                <defs>
                  <linearGradient id="sf-cpu" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--primary)" stopOpacity={0.45} />
                    <stop offset="100%" stopColor="var(--primary)" stopOpacity={0.02} />
                  </linearGradient>
                </defs>
                <Area type="monotone" dataKey="cpu" stroke="var(--primary)" strokeWidth={2} fill="url(#sf-cpu)" isAnimationActive={!reduce} />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  );
}
