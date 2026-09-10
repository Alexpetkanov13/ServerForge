import { useCallback, useEffect, useState } from "react";
import { Navigate, useParams } from "react-router-dom";
import { toast } from "sonner";
import { Download, Puzzle, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { api, errorMessage } from "@/lib/api";
import { supportsMods, supportsPlugins } from "@/lib/providers";
import type { AddonInfo, InstalledAddon } from "@/types";
import { useServerStore } from "@/stores/useServerStore";

export function AddonsPage({ kind }: { kind: "plugins" | "mods" }) {
  const { id } = useParams();
  const server = useServerStore((s) => s.servers.find((x) => x.id === id));
  const [installed, setInstalled] = useState<InstalledAddon[]>([]);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<AddonInfo[]>([]);
  const [searching, setSearching] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const reload = useCallback(() => {
    if (!id) return;
    void api.addons(id, kind).then(setInstalled);
  }, [id, kind]);

  const runSearch = useCallback(
    async (value: string) => {
      if (!server) return;
      const q = value.trim();
      if (!q) {
        setResults([]);
        setSearched(false);
        return;
      }
      setSearching(true);
      try {
        const data =
          kind === "plugins"
            ? await api.searchPlugins(q, server.minecraftVersion, server.provider)
            : await api.searchMods(
                q,
                server.provider === "forge" ? "forge" : server.provider === "neoforge" ? "neoforge" : "fabric",
                server.minecraftVersion,
              );
        setResults(data);
        setSearched(true);
      } catch (err) {
        toast.error(errorMessage(err));
      } finally {
        setSearching(false);
      }
    },
    [kind, server],
  );

  useEffect(() => {
    reload();
  }, [reload]);

  useEffect(() => {
    if (!query.trim()) return;
    const handle = window.setTimeout(() => {
      void runSearch(query);
    }, 350);
    return () => window.clearTimeout(handle);
  }, [query, runSearch]);

  if (!id || !server) return null;
  if (kind === "plugins" && !supportsPlugins(server.provider)) {
    return <Navigate to={`/servers/${server.id}`} replace />;
  }
  if (kind === "mods" && !supportsMods(server.provider)) {
    return <Navigate to={`/servers/${server.id}`} replace />;
  }

  const install = async (item: AddonInfo) => {
    if (!item.compatible) {
      toast.message(`No ${server.minecraftVersion} build is listed for ${item.name}.`);
      return;
    }
    setInstalling(item.id);
    try {
      const file =
        kind === "plugins"
          ? await api.resolvePluginDownload({
              source: item.source,
              id: item.id,
              author: item.author,
              slug: item.slug,
              minecraftVersion: server.minecraftVersion,
              provider: server.provider,
            })
          : (await api.modVersions(item.id, server.provider, server.minecraftVersion))[0];
      if (!file?.downloadUrl) throw new Error("No downloadable version was returned.");
      if (file.requiredDependencies.length) {
        toast.message(`This may require: ${file.requiredDependencies.join(", ")}`);
      }
      await api.installAddon({
        serverId: id,
        kind,
        url: file.downloadUrl,
        fileName: file.fileName || `${item.slug}.jar`,
        label: item.name,
      });
      toast.success(`${item.name} installed into the ${kind} folder.`);
      reload();
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setInstalling(null);
    }
  };

  return (
    <div className="grid gap-6 xl:grid-cols-2">
      <section className="space-y-3">
        <div>
          <h2 className="font-medium">Installed {kind}</h2>
          <p className="text-xs text-muted-foreground">
            Files in this server’s {kind} folder. Disable keeps the jar without loading it.
          </p>
        </div>
        {installed.length === 0 ? (
          <div className="rounded-2xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
            Nothing installed yet. Search on the right and download a compatible build.
          </div>
        ) : (
          installed.map((item) => (
            <div key={item.path} className="flex items-center justify-between rounded-2xl border border-border/80 bg-card/50 p-4 text-sm">
              <div>
                <div className="font-medium">{item.name}</div>
                <div className="text-xs text-muted-foreground">{item.enabled ? "Enabled" : "Disabled"} · {item.fileName}</div>
              </div>
              <div className="flex gap-2">
                <Button variant="outline" onClick={() => void api.toggleAddon(item.path, !item.enabled).then(reload)}>
                  {item.enabled ? "Disable" : "Enable"}
                </Button>
                <Button variant="destructive" onClick={() => void api.removeAddon(item.path).then(reload)}>
                  Remove
                </Button>
              </div>
            </div>
          ))
        )}
      </section>
      <section className="space-y-3">
        <div>
          <h2 className="font-medium">Search {kind} for {server.minecraftVersion}</h2>
          <p className="text-xs text-muted-foreground">
            Results are checked against this server’s version. Compatible {kind} can be downloaded straight into the folder.
          </p>
        </div>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute top-2.5 left-2.5 size-4 text-muted-foreground" />
            <Input
              className="pl-8"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void runSearch(query);
              }}
              placeholder={kind === "plugins" ? "EssentialsX, LuckPerms, Vault…" : "Fabric API, Lithium…"}
            />
          </div>
          <Button onClick={() => void runSearch(query)} disabled={searching}>
            {searching ? "Searching…" : "Search"}
          </Button>
        </div>
        {searched && results.length === 0 ? (
          <p className="text-sm text-muted-foreground">No {kind} matched that search.</p>
        ) : null}
        {results.map((item) => (
          <div key={`${item.source}-${item.id}`} className="rounded-2xl border border-border/80 bg-card/50 p-4 text-sm">
            <div className="flex items-start gap-3">
              {item.iconUrl ? (
                <img src={item.iconUrl} alt="" className="size-10 rounded-lg object-cover" />
              ) : (
                <div className="flex size-10 items-center justify-center rounded-lg bg-muted">
                  <Puzzle className="size-4 text-muted-foreground" />
                </div>
              )}
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <div className="font-medium">{item.name}</div>
                  <span className="rounded-full bg-muted px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground">
                    {item.source}
                  </span>
                  {item.compatible ? (
                    <span className="rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] text-emerald-400">
                      Compatible
                    </span>
                  ) : (
                    <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-[10px] text-amber-400">
                      Not for {server.minecraftVersion}
                    </span>
                  )}
                </div>
                <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{item.description}</p>
                <div className="mt-3 flex items-center justify-between gap-3">
                  <span className="text-xs text-muted-foreground">
                    {item.author} · {item.downloads.toLocaleString()} downloads
                  </span>
                  <Button
                    size="sm"
                    disabled={!item.compatible || installing === item.id}
                    onClick={() => void install(item)}
                  >
                    <Download className="size-3.5" />
                    {installing === item.id ? "Installing…" : "Download"}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        ))}
      </section>
    </div>
  );
}
