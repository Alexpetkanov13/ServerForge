import { Link, useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Activity, FolderInput, MemoryStick, Plus, Server, Wifi } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { StatusDot } from "@/components/server/StatusDot";
import { formatBytes, formatDuration } from "@/lib/format";
import { itemVariants, listVariants } from "@/lib/motion";
import { pickAndImportSharePack } from "@/lib/sharePack";
import { providerLabel } from "@/lib/providers";
import { useAppStore } from "@/stores/useAppStore";
import { useServerStore } from "@/stores/useServerStore";
import banner from "@/assets/banner.webp";

export function DashboardPage() {
  const servers = useServerStore((s) => s.servers);
  const metrics = useServerStore((s) => s.metrics);
  const system = useAppStore((s) => s.system);
  const online = servers.filter((s) => s.status === "online").length;
  const navigate = useNavigate();

  const importPack = async () => {
    const server = await pickAndImportSharePack();
    if (server) navigate(`/servers/${server.id}`);
  };

  return (
    <div className="mx-auto max-w-6xl space-y-6 p-8">
      <section className="relative overflow-hidden rounded-3xl border border-border/70 shadow-[0_20px_80px_oklch(0.2_0.04_50_/_0.45)]">
        <img
          src={banner}
          alt=""
          className="sf-kenburns h-52 w-full object-cover sm:h-64"
          decoding="async"
        />
        <div className="absolute inset-0 bg-gradient-to-r from-black/80 via-black/45 to-transparent" />
        <div className="absolute inset-0 flex flex-col justify-end p-8">
          <p className="text-xs tracking-[0.22em] text-primary uppercase">DHeaven</p>
          <h1 className="mt-1 text-3xl font-semibold tracking-tight text-white">ServerForge</h1>
          <p className="mt-2 max-w-lg text-sm text-white/75">
            Create, share, and command local Minecraft Java servers from one control plane.
          </p>
          <div className="mt-4 flex flex-wrap gap-2">
            <Button variant="outline" className="border-white/20 bg-black/30 text-white hover:bg-black/50" onClick={() => void importPack()}>
              <FolderInput className="size-4" />
              Import .server
            </Button>
            <Button asChild>
              <Link to="/create">
                <Plus className="size-4" />
                Create server
              </Link>
            </Button>
          </div>
        </div>
      </section>
      <motion.div className="grid gap-4 md:grid-cols-4" variants={listVariants} initial="hidden" animate="show">
        <Stat icon={Server} label="Servers" value={String(servers.length)} />
        <Stat icon={Activity} label="Online" value={String(online)} />
        <Stat icon={MemoryStick} label="System RAM" value={system ? `${Math.round(system.totalMemoryMb / 1024)} GB` : "—"} />
        <Stat icon={Wifi} label="Status" value={system?.offline ? "Offline mode" : "Online"} />
      </motion.div>
      <div className="space-y-3">
        <h2 className="text-sm font-medium text-muted-foreground">Servers</h2>
        {servers.length === 0 ? (
          <Card className="border-dashed">
            <CardContent className="flex flex-col items-center gap-3 py-12 text-center">
              <Server className="size-8 text-muted-foreground" />
              <div>
                <div className="font-medium">No servers yet</div>
                <p className="text-sm text-muted-foreground">Create a Paper, Vanilla, or Fabric server in a few clicks.</p>
              </div>
              <div className="flex gap-2">
                <Button variant="outline" onClick={() => void importPack()}>
                  <FolderInput className="size-4" />
                  Import .server
                </Button>
                <Button asChild>
                  <Link to="/create">Create server</Link>
                </Button>
              </div>
            </CardContent>
          </Card>
        ) : (
          <motion.div className="grid gap-3" variants={listVariants} initial="hidden" animate="show">
            {servers.map((server) => {
              const m = metrics[server.id];
              return (
                <motion.div key={server.id} variants={itemVariants} style={{ contentVisibility: "auto" }}>
                  <Link to={`/servers/${server.id}`} className="block">
                    <Card className="sf-card-hover bg-card/80">
                      <CardHeader className="flex flex-row items-center justify-between space-y-0">
                        <CardTitle className="text-base">{server.name}</CardTitle>
                        <StatusDot status={server.status} />
                      </CardHeader>
                      <CardContent className="flex flex-wrap gap-6 text-sm text-muted-foreground">
                        <span>
                          {providerLabel(server.provider)} {server.minecraftVersion}
                        </span>
                        <span>localhost:{server.port}</span>
                        {m ? (
                          <>
                            <span>CPU {m.cpuPercent.toFixed(0)}%</span>
                            <span>RAM {formatBytes(m.memoryBytes)}</span>
                            <span>Up {formatDuration(m.uptimeSeconds)}</span>
                          </>
                        ) : null}
                      </CardContent>
                    </Card>
                  </Link>
                </motion.div>
              );
            })}
          </motion.div>
        )}
      </div>
    </div>
  );
}

function Stat({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string;
  icon: typeof Server;
}) {
  return (
    <motion.div variants={itemVariants}>
      <Card className="sf-card-hover bg-card/70">
        <CardContent className="p-5">
          <div className="flex items-center justify-between">
            <div className="text-xs uppercase tracking-wide text-muted-foreground">{label}</div>
            <Icon className="size-4 text-primary/80" />
          </div>
          <div className="mt-1 text-2xl font-semibold">{value}</div>
        </CardContent>
      </Card>
    </motion.div>
  );
}
