import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { motion } from "framer-motion";
import { FolderInput, FolderSearch, Play, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { StatusDot } from "@/components/server/StatusDot";
import { api, errorMessage } from "@/lib/api";
import { itemVariants, listVariants } from "@/lib/motion";
import { pickAndImportSharePack } from "@/lib/sharePack";
import { useServerStore } from "@/stores/useServerStore";
import { Checkbox } from "@/components/ui/checkbox";
import { DeleteServerButton } from "@/components/server/DeleteServerButton";
import { PageHeader } from "@/components/layout/PageHeader";

export function ServersPage() {
  const servers = useServerStore((s) => s.servers);
  const refresh = useServerStore((s) => s.refresh);
  const navigate = useNavigate();
  const [importing, setImporting] = useState(false);

  const scan = async () => {
    const selected = await open({ directory: true, title: "Scan for servers" });
    if (typeof selected !== "string") return;
    setImporting(true);
    try {
      const found = await api.scan(selected);
      if (!found.length) {
        toast.message("No servers found in that folder.");
        return;
      }
      for (const item of found) {
        await api.importServer(item);
      }
      await refresh();
      toast.success(`Imported ${found.length} server${found.length === 1 ? "" : "s"}.`);
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setImporting(false);
    }
  };

  const importPack = async () => {
    setImporting(true);
    try {
      const server = await pickAndImportSharePack();
      if (server) navigate(`/servers/${server.id}`);
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="mx-auto max-w-6xl space-y-6 p-8">
      <PageHeader title="Servers" subtitle="Start, stop, and manage every local server.">
        <Button variant="outline" onClick={() => void scan()} disabled={importing}>
          <FolderSearch className="size-4" />
          Scan for servers
        </Button>
        <Button variant="outline" onClick={() => void importPack()} disabled={importing}>
          <FolderInput className="size-4" />
          Import .server
        </Button>
        <Button asChild>
          <Link to="/create">Create server</Link>
        </Button>
      </PageHeader>
      {servers.length === 0 ? (
        <Card className="border-dashed">
          <CardContent className="py-12 text-center text-sm text-muted-foreground">
            No servers yet. Create one or import a .server pack.
          </CardContent>
        </Card>
      ) : (
        <motion.div className="grid gap-3" variants={listVariants} initial="hidden" animate="show">
          {servers.map((server) => (
            <motion.div key={server.id} variants={itemVariants} style={{ contentVisibility: "auto" }}>
              <Card className="sf-card-hover bg-card/80">
                <CardContent className="flex items-center justify-between gap-4 p-5">
                  <button className="text-left" onClick={() => navigate(`/servers/${server.id}`)}>
                    <div className="font-medium">{server.name}</div>
                    <div className="mt-1 flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
                      <StatusDot status={server.status} />
                      <span>
                        {server.provider} {server.minecraftVersion}
                      </span>
                      <span>127.0.0.1:{server.port}</span>
                    </div>
                  </button>
                  <div className="flex items-center gap-3">
                    <label className="flex items-center gap-2 text-xs text-muted-foreground">
                      <Checkbox
                        checked={server.autoStart === 1}
                        onCheckedChange={(v) => {
                          void api.setAutoStart(server.id, Boolean(v)).then(() => refresh());
                        }}
                      />
                      Auto-start
                    </label>
                    {server.status === "online" || server.status === "starting" || server.status === "stopping" ? (
                      <Button
                        variant="outline"
                        onClick={() =>
                          void api
                            .stop(server.id)
                            .then(refresh)
                            .then(() => toast.success("Server stopped"))
                            .catch((e) => toast.error(errorMessage(e)))
                        }
                      >
                        <Square className="size-4" />
                        Stop
                      </Button>
                    ) : (
                      <Button onClick={() => void api.start(server.id).then(refresh).catch((e) => toast.error(errorMessage(e)))}>
                        <Play className="size-4" />
                        Start
                      </Button>
                    )}
                    <DeleteServerButton id={server.id} name={server.name} variant="ghost" />
                    <Button variant="ghost" onClick={() => navigate(`/servers/${server.id}`)}>
                      Manage
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </motion.div>
          ))}
        </motion.div>
      )}
    </div>
  );
}
