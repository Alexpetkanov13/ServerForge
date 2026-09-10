import { memo, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import {
  ArrowLeft,
  ArrowRight,
  Box,
  Check,
  Cpu,
  Feather,
  FolderOpen,
  Hammer,
  Import,
  Layers,
  LoaderCircle,
  MemoryStick,
  ScrollText,
  Server,
  Settings2,
  Sparkles,
  Zap,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { Slider } from "@/components/ui/slider";
import { api, errorDetails, errorMessage } from "@/lib/api";
import { pickAndImportSharePack } from "@/lib/sharePack";
import { formatBytes, formatGb } from "@/lib/format";
import { useAppStore } from "@/stores/useAppStore";
import { useServerStore } from "@/stores/useServerStore";
import { useWizardStore } from "@/stores/useWizardStore";
import { cn } from "@/lib/utils";
import type {
  BuildInfo,
  CompatibilityInfo,
  DiskInfo,
  InstallationProgress,
  MinecraftVersionInfo,
  ProviderInfo,
} from "@/types";

const STEPS = ["Name", "Location", "Platform", "Version", "Java", "Memory", "EULA", "Review", "Install"];

const STEP_HINTS = [
  "Give your world a name worth remembering.",
  "Choose where the server files will live.",
  "Pick the engine — vanilla, plugins or mods.",
  "Pin the Minecraft version and build.",
  "Choose the Java runtime that powers it.",
  "Allocate RAM, ports and world options.",
  "Accept the Minecraft EULA to continue.",
  "Double-check everything before forging.",
  "Watch your server come to life.",
];

function providerVisual(id: string) {
  const key = id.toLowerCase();
  if (key.includes("paper")) return { Icon: Feather, glow: "from-sky-500/25 via-sky-500/5 to-transparent", ring: "group-data-[sel=true]:border-sky-400/60", chip: "bg-sky-500/15 text-sky-300" };
  if (key.includes("purpur")) return { Icon: Sparkles, glow: "from-fuchsia-500/25 via-fuchsia-500/5 to-transparent", ring: "group-data-[sel=true]:border-fuchsia-400/60", chip: "bg-fuchsia-500/15 text-fuchsia-300" };
  if (key.includes("folia")) return { Icon: Zap, glow: "from-amber-500/25 via-amber-500/5 to-transparent", ring: "group-data-[sel=true]:border-amber-400/60", chip: "bg-amber-500/15 text-amber-300" };
  if (key.includes("fabric")) return { Icon: Layers, glow: "from-orange-500/25 via-orange-500/5 to-transparent", ring: "group-data-[sel=true]:border-orange-400/60", chip: "bg-orange-500/15 text-orange-300" };
  if (key.includes("forge") || key.includes("neoforge")) return { Icon: Hammer, glow: "from-red-500/25 via-red-500/5 to-transparent", ring: "group-data-[sel=true]:border-red-400/60", chip: "bg-red-500/15 text-red-300" };
  if (key.includes("spigot") || key.includes("bukkit")) return { Icon: Server, glow: "from-emerald-500/25 via-emerald-500/5 to-transparent", ring: "group-data-[sel=true]:border-emerald-400/60", chip: "bg-emerald-500/15 text-emerald-300" };
  return { Icon: Box, glow: "from-zinc-500/25 via-zinc-500/5 to-transparent", ring: "group-data-[sel=true]:border-primary", chip: "bg-muted text-muted-foreground" };
}

const ProviderCard = memo(function ProviderCard({
  provider,
  selected,
  index,
  onPick,
}: {
  provider: ProviderInfo;
  selected: boolean;
  index: number;
  onPick: () => void;
}) {
  const { Icon, glow, ring, chip } = providerVisual(provider.id);
  return (
    <motion.button
      type="button"
      onClick={onPick}
      data-sel={selected}
      initial={{ opacity: 0, y: 22, scale: 0.97 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      transition={{ duration: 0.35, delay: Math.min(index * 0.06, 0.36), ease: [0.22, 1, 0.36, 1] }}
      whileHover={{ y: -4, scale: 1.015 }}
      whileTap={{ scale: 0.975 }}
      className={cn("group relative text-left outline-none", selected && "z-10")}
    >
      {/* hover glow */}
      <span
        className={cn(
          "pointer-events-none absolute -inset-px rounded-2xl bg-gradient-to-b opacity-0 blur-xl transition-opacity duration-300 group-hover:opacity-100",
          glow,
        )}
      />
      <Card
        className={cn(
          "relative overflow-hidden transition-all duration-300 group-hover:shadow-[0_18px_50px_-16px_oklch(0_0_0_/_0.7)]",
          ring,
          selected
            ? "border-primary/70 shadow-[0_0_0_1px_var(--primary),0_0_32px_oklch(0.78_0.15_55_/_0.3)]"
            : "hover:border-foreground/25",
        )}
      >
        {/* top sheen for selected */}
        <AnimatePresence>
          {selected && (
            <motion.span
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-primary to-transparent"
            />
          )}
        </AnimatePresence>
        <CardContent className="space-y-2.5 p-5">
          <div className="flex items-center justify-between">
            <span className={cn("grid size-9 place-items-center rounded-xl border border-border bg-muted/60 transition-transform duration-300 group-hover:scale-110 group-hover:rotate-3", selected && "border-primary/50 bg-primary/10")}>
              <Icon className="size-4.5" />
            </span>
            <AnimatePresence mode="popLayout">
              {selected ? (
                <motion.span
                  key="check"
                  initial={{ scale: 0, rotate: -90 }}
                  animate={{ scale: 1, rotate: 0 }}
                  exit={{ scale: 0, rotate: 90 }}
                  transition={{ type: "spring", stiffness: 500, damping: 26 }}
                  className="grid size-6 place-items-center rounded-full bg-primary text-primary-foreground shadow-[0_0_16px_oklch(0.78_0.15_55_/_0.5)]"
                >
                  <Check className="size-3.5" strokeWidth={3} />
                </motion.span>
              ) : (
                <motion.span
                  key="kind"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className={cn("rounded-full px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider", chip)}
                >
                  {provider.supportsPlugins ? "Plugins" : provider.supportsMods ? "Mods" : "Vanilla"}
                </motion.span>
              )}
            </AnimatePresence>
          </div>
          <div className="text-sm font-semibold uppercase tracking-wide">{provider.name}</div>
          <p className="min-h-10 text-sm leading-snug text-muted-foreground">{provider.description}</p>
          <p className="truncate text-xs text-muted-foreground/80">{provider.recommendedUse}</p>
          <p className="text-xs font-medium text-foreground/70">⚡ {provider.performance}</p>
        </CardContent>
      </Card>
    </motion.button>
  );
});

const VersionRow = memo(function VersionRow({
  v,
  active,
  onPick,
  index,
}: {
  v: MinecraftVersionInfo;
  active: boolean;
  onPick: () => void;
  index: number;
}) {
  return (
    <motion.button
      type="button"
      onClick={onPick}
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.22, delay: Math.min(index * 0.015, 0.3) }}
      whileTap={{ scale: 0.985 }}
      className={cn(
        "flex items-center justify-between rounded-lg border px-3 py-2 text-sm transition-all duration-200",
        active
          ? "border-primary bg-primary/[0.07] shadow-[0_0_18px_oklch(0.78_0.15_55_/_0.25)]"
          : "border-border hover:border-foreground/25 hover:bg-muted/50",
      )}
    >
      <span className="flex items-center gap-2 font-mono">
        {active && <motion.span layoutId="version-dot" className="size-1.5 rounded-full bg-primary" />}
        {v.id}
      </span>
      <span className="text-xs text-muted-foreground">
        {v.latest ? "Latest · " : ""} {v.recommended ? "✦ Recommended" : v.channel}
      </span>
    </motion.button>
  );
});

export function CreateServerPage() {
  const { step, request, setStep, patch, reset } = useWizardStore();
  const javas = useAppStore((s) => s.javas);
  const system = useAppStore((s) => s.system);
  const refresh = useServerStore((s) => s.refresh);
  const navigate = useNavigate();
  const reduce = useReducedMotion();
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [versions, setVersions] = useState<MinecraftVersionInfo[]>([]);
  const [builds, setBuilds] = useState<BuildInfo[]>([]);
  const [compat, setCompat] = useState<CompatibilityInfo | null>(null);
  const [disk, setDisk] = useState<DiskInfo | null>(null);
  const [nameError, setNameError] = useState("");
  const [search, setSearch] = useState("");
  const [install, setInstall] = useState<InstallationProgress | null>(null);
  const [busy, setBusy] = useState(false);
  const dirRef = useRef(1);

  useEffect(() => {
    void api.providerCatalog().then(setProviders);
  }, []);

  useEffect(() => {
    if (!request.name || request.path) return;
    void api.defaultPath(request.name).then((path) => patch({ path }));
  }, [request.name, request.path, patch]);

  useEffect(() => {
    if (!request.path) return;
    const t = setTimeout(() => {
      void api.inspectDirectory(request.path).then(setDisk).catch(() => setDisk(null));
    }, 250);
    return () => clearTimeout(t);
  }, [request.path]);

  useEffect(() => {
    if (!request.provider) return;
    void api.versions(request.provider).then((list) => {
      setVersions(list);
      const recommended = list.find((v) => v.recommended) ?? list.find((v) => v.channel === "stable") ?? list[0];
      if (recommended && !request.minecraftVersion) {
        patch({ minecraftVersion: recommended.id });
      }
    }).catch((err) => toast.error(errorMessage(err)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [request.provider]);

  useEffect(() => {
    if (!request.provider || !request.minecraftVersion) return;
    void Promise.all([
      api.builds(request.provider, request.minecraftVersion),
      api.compatibility(request.provider, request.minecraftVersion),
    ]).then(([b, c]) => {
      setBuilds(b);
      setCompat(c);
      const rec = b.find((x) => x.recommended) ?? b[0];
      if (rec) {
        patch({
          build: rec.id,
          loaderVersion: rec.loaderVersion,
          installerVersion: rec.installerVersion,
        });
      }
    }).catch((err) => toast.error(errorMessage(err)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [request.provider, request.minecraftVersion]);

  useEffect(() => {
    const unlisten = listen<InstallationProgress>("installation-progress", (e) => setInstall(e.payload));
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  const filteredVersions = useMemo(
    () => versions.filter((v) => v.id.toLowerCase().includes(search.toLowerCase())),
    [versions, search],
  );
  const topBuilds = useMemo(() => builds.slice(0, 20), [builds]);
  const progress = useMemo(() => ((step + 1) / STEPS.length) * 100, [step]);

  const go = (next: number) => {
    dirRef.current = next > step ? 1 : -1;
    setStep(Math.max(0, Math.min(next, STEPS.length - 1)));
  };

  const next = async () => {
    if (step === 0) {
      try {
        await api.validateName(request.name);
        setNameError("");
      } catch (err) {
        setNameError(errorMessage(err));
        return;
      }
    }
    go(step + 1);
  };

  const create = async () => {
    setBusy(true);
    go(8);
    try {
      const id = await api.createServer({
        ...request,
        motd: request.motd || request.name,
      });
      await refresh();
      toast.success("Server is ready.");
      reset();
      navigate(`/servers/${id}`);
    } catch (err) {
      const details = errorDetails(err);
      toast.error(details.message || errorMessage(err));
      setInstall((prev) => prev);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="relative mx-auto max-w-4xl space-y-6 p-8">
      {/* ambient orbs */}
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10 overflow-hidden">
        <motion.div
          animate={reduce ? undefined : { x: [0, 30, 0], y: [0, -20, 0] }}
          transition={{ duration: 14, repeat: Infinity, ease: "easeInOut" }}
          className="absolute -top-24 left-1/4 size-72 rounded-full bg-primary/[0.08] blur-3xl"
        />
        <motion.div
          animate={reduce ? undefined : { x: [0, -24, 0], y: [0, 18, 0] }}
          transition={{ duration: 17, repeat: Infinity, ease: "easeInOut" }}
          className="absolute right-0 top-40 size-64 rounded-full bg-emerald-500/[0.07] blur-3xl"
        />
      </div>

      <motion.div initial={reduce ? false : { opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.35 }}>
        <h1 className="bg-gradient-to-br from-foreground to-foreground/60 bg-clip-text text-2xl font-semibold tracking-tight">
          Create server
        </h1>
        <AnimatePresence mode="wait">
          <motion.p
            key={step}
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.2 }}
            className="text-sm text-muted-foreground"
          >
            Step {step + 1} of {STEPS.length} — {STEP_HINTS[step]}
          </motion.p>
        </AnimatePresence>
        <Button
          variant="outline"
          className="mt-3 transition-all hover:-translate-y-0.5 hover:shadow-lg"
          onClick={async () => {
            const server = await pickAndImportSharePack();
            if (server) navigate(`/servers/${server.id}`);
          }}
        >
          <Import className="size-4" />
          Import a .server file instead
        </Button>
      </motion.div>

      {/* Stepper */}
      <div className="space-y-2.5">
        <div className="flex flex-wrap gap-1.5 text-xs">
          {STEPS.map((label, i) => {
            const active = i === step;
            const done = i < step;
            return (
              <button
                key={label}
                onClick={() => i < step && go(i)}
                disabled={i >= step && step < 8}
                className={cn(
                  "relative rounded-full px-2.5 py-1 transition-all duration-200",
                  active && "text-primary-foreground",
                  done && "bg-emerald-500/15 text-emerald-400 hover:bg-emerald-500/25",
                  !active && !done && "bg-muted text-muted-foreground",
                  done && "cursor-pointer",
                )}
              >
                {active && (
                  <motion.span
                    layoutId="wizard-step-pill"
                    className="absolute inset-0 rounded-full bg-primary shadow-[0_0_16px_oklch(0.78_0.15_55_/_0.35)]"
                    transition={{ type: "spring", stiffness: 480, damping: 36 }}
                  />
                )}
                <span className="relative flex items-center gap-1">
                  {done && <Check className="size-3" strokeWidth={3} />}
                  {label}
                </span>
              </button>
            );
          })}
        </div>
        <div className="h-1 overflow-hidden rounded-full bg-muted">
          <motion.div
            className="h-full rounded-full bg-gradient-to-r from-primary via-amber-400 to-emerald-400"
            animate={{ width: `${progress}%` }}
            transition={{ type: "spring", stiffness: 120, damping: 22 }}
          />
        </div>
      </div>

      {/* Step body */}
      <div className="relative min-h-[320px]">
        <AnimatePresence mode="wait" custom={dirRef.current}>
          <motion.div
            key={step}
            custom={dirRef.current}
            initial={reduce ? { opacity: 0 } : { opacity: 0, x: 48 * dirRef.current, scale: 0.985 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={reduce ? { opacity: 0 } : { opacity: 0, x: -48 * dirRef.current, scale: 0.985 }}
            transition={{ duration: 0.26, ease: [0.22, 1, 0.36, 1] }}
          >
            {step === 0 && (
              <Field icon={<Server className="size-4" />} title="Server name" hint="This becomes the folder name, tab title and default MOTD.">
                <Input
                  value={request.name}
                  onChange={(e) => patch({ name: e.target.value })}
                  onKeyDown={(e) => e.key === "Enter" && void next()}
                  placeholder="My Survival Server"
                  autoFocus
                  className="h-11 text-base transition-shadow focus-visible:shadow-[0_0_24px_oklch(0.78_0.15_55_/_0.25)]"
                />
                <AnimatePresence>
                  {nameError ? (
                    <motion.p initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: "auto" }} exit={{ opacity: 0, height: 0 }} className="overflow-hidden text-sm text-destructive">
                      {nameError}
                    </motion.p>
                  ) : null}
                </AnimatePresence>
              </Field>
            )}
            {step === 1 && (
              <Field icon={<FolderOpen className="size-4" />} title="Server location" hint="Everything for this server lives in one folder.">
                <div className="flex gap-2">
                  <Input value={request.path} onChange={(e) => patch({ path: e.target.value })} className="font-mono text-sm" />
                  <Button
                    variant="outline"
                    className="shrink-0 transition-all hover:-translate-y-0.5"
                    onClick={async () => {
                      const selected = await open({ directory: true });
                      if (typeof selected === "string") patch({ path: selected });
                    }}
                  >
                    <FolderOpen className="size-4" />
                    Browse
                  </Button>
                </div>
                <AnimatePresence>
                  {disk ? (
                    <motion.ul initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} className="space-y-1 text-sm text-muted-foreground">
                      <li className={disk.writable ? "text-emerald-400" : "text-destructive"}>{disk.writable ? "✓ Directory is writable" : "✕ Directory is not writable"}</li>
                      <li>✓ {formatBytes(disk.availableBytes)} available</li>
                      <li className={disk.containsServer ? "text-amber-300" : ""}>
                        {disk.containsServer ? "⚠ Folder looks like it already contains a server" : "✓ Folder does not contain another server"}
                      </li>
                      <li>✓ A shareable .server file will be saved inside this folder</li>
                    </motion.ul>
                  ) : null}
                </AnimatePresence>
              </Field>
            )}
            {step === 2 && (
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <span className="grid size-8 place-items-center rounded-lg bg-primary/10 text-primary"><Cpu className="size-4" /></span>
                  <div>
                    <h2 className="text-lg font-medium leading-tight">Choose your platform</h2>
                    <p className="text-sm text-muted-foreground">Each engine glows differently — pick the one that fits your playstyle.</p>
                  </div>
                </div>
                {providers.length === 0 ? (
                  <div className="grid gap-3 sm:grid-cols-2">
                    {[0, 1, 2, 3].map((i) => (
                      <motion.div key={i} animate={reduce ? undefined : { opacity: [0.4, 0.8, 0.4] }} transition={{ duration: 1.4, repeat: Infinity, delay: i * 0.15 }} className="h-36 rounded-2xl border border-border bg-muted/40" />
                    ))}
                  </div>
                ) : (
                  <div className="grid gap-3 sm:grid-cols-2">
                    {providers.map((p, i) => (
                      <ProviderCard key={p.id} provider={p} index={i} selected={request.provider === p.id} onPick={() => patch({ provider: p.id, minecraftVersion: "" })} />
                    ))}
                  </div>
                )}
              </div>
            )}
            {step === 3 && (
              <div className="space-y-4">
                <Input placeholder="Search versions…" value={search} onChange={(e) => setSearch(e.target.value)} className="transition-shadow focus-visible:shadow-[0_0_18px_oklch(0.78_0.15_55_/_0.2)]" />
                <AnimatePresence>
                  {compat ? (
                    <motion.div initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0 }} className="relative overflow-hidden rounded-xl border border-emerald-500/25 bg-emerald-500/[0.06] p-3 text-sm">
                      <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-emerald-400 to-transparent" />
                      <div className="font-medium">{compat.provider} · Minecraft {compat.minecraftVersion}</div>
                      <div className="mt-1 text-muted-foreground">✓ Supported · {compat.stableBuild ? "Stable build available" : "Latest build"} · Java {compat.javaMajor} recommended</div>
                    </motion.div>
                  ) : null}
                </AnimatePresence>
                <div className="grid max-h-72 gap-2 overflow-auto pr-1">
                  {filteredVersions.map((v, i) => (
                    <VersionRow key={v.id} v={v} index={i} active={request.minecraftVersion === v.id} onPick={() => patch({ minecraftVersion: v.id })} />
                  ))}
                </div>
                {topBuilds.length > 0 && (
                  <div className="space-y-2">
                    <Label>Build / loader</Label>
                    <div className="grid max-h-48 gap-2 overflow-auto pr-1">
                      {topBuilds.map((b) => (
                        <button
                          key={b.id}
                          onClick={() => patch({ build: b.id, loaderVersion: b.loaderVersion, installerVersion: b.installerVersion })}
                          className={cn("rounded-lg border px-3 py-2 text-left text-sm transition-all duration-200", request.build === b.id ? "border-primary bg-primary/[0.07] shadow-[0_0_16px_oklch(0.78_0.15_55_/_0.25)]" : "border-border hover:border-foreground/25")}
                        >
                          {b.label}
                          {b.recommended && <span className="ml-2 text-xs text-primary">✦ recommended</span>}
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
            {step === 4 && (
              <div className="space-y-3">
                {javas.length === 0 ? (
                  <p className="rounded-xl border border-dashed border-border p-4 text-sm text-muted-foreground">
                    Java {compat?.javaMajor ?? 21} was not detected. Install Eclipse Temurin or select a java executable later.
                  </p>
                ) : (
                  javas.map((j, i) => (
                    <motion.button
                      key={j.path}
                      type="button"
                      onClick={() => patch({ javaPath: j.path })}
                      initial={{ opacity: 0, y: 10 }}
                      animate={{ opacity: 1, y: 0 }}
                      transition={{ delay: Math.min(i * 0.05, 0.3) }}
                      whileHover={{ y: -2 }}
                      whileTap={{ scale: 0.99 }}
                      className={cn("block w-full rounded-xl border p-3 text-left text-sm transition-all duration-200", request.javaPath === j.path ? "border-primary bg-primary/[0.06] shadow-[0_0_20px_oklch(0.78_0.15_55_/_0.25)]" : "border-border hover:border-foreground/25")}
                    >
                      <div className="flex items-center gap-2 font-medium">
                        {request.javaPath === j.path && <Check className="size-4 text-primary" strokeWidth={3} />}
                        Java {j.major} · {j.vendor}
                      </div>
                      <div className="mt-0.5 font-mono text-xs text-muted-foreground">{j.path}</div>
                    </motion.button>
                  ))
                )}
              </div>
            )}
            {step === 5 && (
              <div className="space-y-4">
                <div className="flex items-center gap-2">
                  <span className="grid size-8 place-items-center rounded-lg bg-primary/10 text-primary"><MemoryStick className="size-4" /></span>
                  <h2 className="text-lg font-medium">Memory & network</h2>
                </div>
                <div className="flex flex-wrap gap-2">
                  {[["Light", 2048], ["Standard", 4096], ["Large", 6144], ["Heavy", 8192]].map(([label, mb]) => {
                    const active = request.memoryMaxMb === Number(mb);
                    return (
                      <Button key={label as string} variant={active ? "default" : "outline"} onClick={() => patch({ memoryMinMb: Number(mb) / 2, memoryMaxMb: Number(mb) })} className={cn("transition-all", active && "shadow-[0_0_18px_oklch(0.78_0.15_55_/_0.35)]")}>
                        {label as string}
                      </Button>
                    );
                  })}
                </div>
                <Slider
                  min={1024}
                  max={Math.max(system ? system.totalMemoryMb - 2048 : 16384, 2048)}
                  step={512}
                  value={[request.memoryMaxMb]}
                  onValueChange={([v]) => patch({ memoryMaxMb: v, memoryMinMb: Math.min(request.memoryMinMb, v) })}
                />
                <p className="text-sm text-muted-foreground">
                  System RAM: {system ? formatGb(system.totalMemoryMb) : "—"} · Allocated: <span className="font-semibold text-foreground">{formatGb(request.memoryMaxMb)}</span>
                </p>
                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <Label>Port</Label>
                    <Input type="number" value={request.port} onChange={(e) => patch({ port: Number(e.target.value) })} />
                  </div>
                  <div>
                    <Label>MOTD</Label>
                    <Input value={request.motd ?? ""} onChange={(e) => patch({ motd: e.target.value })} />
                  </div>
                </div>
                <label className="flex cursor-pointer items-center gap-2 text-sm">
                  <Checkbox checked={request.generateWorld} onCheckedChange={(v) => patch({ generateWorld: Boolean(v) })} />
                  Initialize world after install
                </label>
                <label className="flex cursor-pointer items-center gap-2 text-sm">
                  <Checkbox checked={request.startAfterInstall} onCheckedChange={(v) => patch({ startAfterInstall: Boolean(v) })} />
                  Start server after installation
                </label>
              </div>
            )}
            {step === 6 && (
              <motion.div initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} className="space-y-4 rounded-2xl border border-border bg-card/60 p-5 text-sm backdrop-blur">
                <div className="flex items-center gap-2">
                  <ScrollText className="size-5 text-primary" />
                  <h2 className="text-lg font-medium">One last legal bit</h2>
                </div>
                <p className="leading-relaxed text-muted-foreground">
                  Minecraft servers require accepting the{" "}
                  <a className="text-foreground underline decoration-primary/60 underline-offset-2" href="https://aka.ms/MinecraftEULA" target="_blank" rel="noreferrer">
                    Minecraft EULA
                  </a>
                  . ServerForge will not write <code className="rounded bg-muted px-1 font-mono text-xs">eula=true</code> until you confirm here.
                </p>
                <motion.label whileTap={{ scale: 0.99 }} className={cn("flex cursor-pointer items-center gap-3 rounded-xl border p-3 transition-all", request.eulaAccepted ? "border-emerald-500/50 bg-emerald-500/[0.07]" : "border-border hover:border-foreground/25")}>
                  <Checkbox checked={request.eulaAccepted} onCheckedChange={(v) => patch({ eulaAccepted: Boolean(v) })} />
                  <span className="font-medium">I accept the Minecraft EULA</span>
                  {request.eulaAccepted && <Check className="ml-auto size-4 text-emerald-400" strokeWidth={3} />}
                </motion.label>
              </motion.div>
            )}
            {step === 7 && (
              <motion.div initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }}>
                <Card className="overflow-hidden">
                  <div className="h-1 bg-gradient-to-r from-primary via-amber-400 to-emerald-400" />
                  <CardContent className="grid gap-2 p-5 text-sm">
                    <Row k="Name" v={request.name} />
                    <Row k="Location" v={request.path} mono />
                    <Row k="Platform" v={request.provider} />
                    <Row k="Minecraft" v={request.minecraftVersion} />
                    <Row k="Build" v={request.build ?? "Recommended"} />
                    <Row k="Java" v={request.javaPath ?? "Auto-detect"} mono />
                    <Row k="Memory" v={formatGb(request.memoryMaxMb)} />
                    <Row k="Port" v={String(request.port)} />
                  </CardContent>
                </Card>
              </motion.div>
            )}
            {step === 8 && (
              <div className="space-y-4">
                <div className="flex items-center gap-3 text-lg font-medium">
                  <AnimatePresence mode="wait">
                    {busy ? (
                      <motion.span key="spin" exit={{ scale: 0, opacity: 0 }} className="grid size-9 place-items-center rounded-full bg-primary/10">
                        <LoaderCircle className="size-5 animate-spin text-primary" />
                      </motion.span>
                    ) : (
                      <motion.span key="done" initial={{ scale: 0, rotate: -90 }} animate={{ scale: 1, rotate: 0 }} transition={{ type: "spring", stiffness: 400, damping: 20 }} className="grid size-9 place-items-center rounded-full bg-emerald-500/15">
                        <Check className="size-5 text-emerald-400" strokeWidth={3} />
                      </motion.span>
                    )}
                  </AnimatePresence>
                  <span>
                    Creating {request.name || "server"}
                    {busy && (
                      <motion.span animate={reduce ? undefined : { opacity: [1, 0.3, 1] }} transition={{ duration: 1.2, repeat: Infinity }} className="text-muted-foreground">…</motion.span>
                    )}
                  </span>
                  <span className="ml-auto font-mono text-sm text-muted-foreground">{Math.round(install?.percent ?? (busy ? 8 : 100))}%</span>
                </div>
                <div className="relative">
                  <Progress value={install?.percent ?? (busy ? 8 : 100)} className="h-2.5" />
                  {busy && <span className="sf-progress-shimmer pointer-events-none absolute inset-0 rounded-full" />}
                </div>
                <ul className="max-h-48 space-y-1 overflow-auto rounded-xl border border-border/60 bg-muted/30 p-3 text-sm text-muted-foreground">
                  <AnimatePresence initial={false}>
                    {(install?.logs ?? []).map((line) => (
                      <motion.li key={line} initial={{ opacity: 0, x: -8 }} animate={{ opacity: 1, x: 0 }} className="flex items-center gap-1.5">
                        <Check className="size-3.5 shrink-0 text-emerald-400" /> {line}
                      </motion.li>
                    ))}
                  </AnimatePresence>
                  {(install?.logs ?? []).length === 0 && <li className="text-xs">Preparing download…</li>}
                </ul>
                {install?.error ? (
                  <div className="rounded-xl border border-destructive/40 bg-destructive/[0.06] p-3 text-sm">
                    <div className="font-medium">{install.error.title}</div>
                    <p>{install.error.message}</p>
                    <ul className="mt-2 list-disc pl-4 text-muted-foreground">
                      {install.error.causes.map((c) => (
                        <li key={c}>{c}</li>
                      ))}
                    </ul>
                    <details className="mt-2">
                      <summary className="cursor-pointer">Technical details</summary>
                      <pre className="mt-2 overflow-auto text-xs">{install.error.technical}</pre>
                    </details>
                  </div>
                ) : null}
              </div>
            )}
          </motion.div>
        </AnimatePresence>
      </div>

      {step < 8 && (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="flex items-center justify-between">
          <Button variant="ghost" onClick={() => go(step - 1)} disabled={step === 0} className="transition-all hover:-translate-x-0.5">
            <ArrowLeft className="size-4" /> Back
          </Button>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Settings2 className="size-3.5" />
            {step + 1} / {STEPS.length}
          </div>
          {step === 7 ? (
            <motion.div whileHover={{ scale: 1.02 }} whileTap={{ scale: 0.97 }}>
              <Button onClick={() => void create()} disabled={!request.eulaAccepted || busy} className="shadow-[0_0_24px_oklch(0.78_0.15_55_/_0.35)]">
                <Sparkles className="size-4" /> Forge server
              </Button>
            </motion.div>
          ) : (
            <Button onClick={() => void next()} disabled={(step === 0 && !request.name.trim()) || (step === 6 && !request.eulaAccepted)} className="group">
              Continue <ArrowRight className="size-4 transition-transform group-hover:translate-x-0.5" />
            </Button>
          )}
        </motion.div>
      )}
    </div>
  );
}

function Field({ title, hint, icon, children }: { title: string; hint?: string; icon?: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2.5">
        {icon && <span className="grid size-8 place-items-center rounded-lg bg-primary/10 text-primary">{icon}</span>}
        <div>
          <h2 className="text-lg font-medium leading-tight">{title}</h2>
          {hint && <p className="text-sm text-muted-foreground">{hint}</p>}
        </div>
      </div>
      {children}
    </div>
  );
}

function Row({ k, v, mono }: { k: string; v: string; mono?: boolean }) {
  return (
    <div className="flex justify-between gap-4">
      <span className="shrink-0 text-muted-foreground">{k}</span>
      <span className={cn("truncate text-right font-medium", mono && "font-mono text-xs")}>{v}</span>
    </div>
  );
}
