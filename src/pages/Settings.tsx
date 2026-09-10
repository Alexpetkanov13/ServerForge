import { motion, useReducedMotion } from "framer-motion";
import { ArrowDownToLine, Check } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api, errorMessage } from "@/lib/api";
import { PageHeader } from "@/components/layout/PageHeader";
import { useAppStore } from "@/stores/useAppStore";
import { useUpdateStore } from "@/stores/useUpdateStore";
import { THEMES, normalizeTheme } from "@/lib/themes";
import { cn } from "@/lib/utils";

export function SettingsPage() {
  const settings = useAppStore((s) => s.settings);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const appVersion = useAppStore((s) => s.system?.appVersion);
  const updatePhase = useUpdateStore((s) => s.phase);
  const checkNow = useUpdateStore((s) => s.checkNow);
  const reduce = useReducedMotion();
  if (!settings) {
    return <div className="p-8 text-sm text-muted-foreground">Settings will appear once ServerForge finishes loading.</div>;
  }
  const patch = (partial: Partial<typeof settings>) => void saveSettings({ ...settings, ...partial });
  return (
    <div className="mx-auto max-w-2xl space-y-8 p-8">
      <PageHeader title="Settings" subtitle="Appearance, backups, and developer tools." />
      <section className="space-y-3">
        <h2 className="font-medium">General</h2>
        <Label>Default servers directory</Label>
        <div className="flex gap-2">
          <Input value={settings.defaultServersDir} onChange={(e) => patch({ defaultServersDir: e.target.value })} />
          <Button
            variant="outline"
            onClick={async () => {
              const selected = await open({ directory: true });
              if (typeof selected === "string") patch({ defaultServersDir: selected });
            }}
          >
            Browse
          </Button>
        </div>
      </section>
      <section className="space-y-3">
        <h2 className="font-medium">Appearance</h2>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          {THEMES.map((t, i) => {
            const active = normalizeTheme(settings.theme) === t.id;
            return (
              <motion.button
                key={t.id}
                type="button"
                onClick={() => patch({ theme: t.id })}
                initial={reduce ? false : { opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.28, delay: i * 0.05 }}
                whileHover={reduce ? undefined : { y: -3 }}
                whileTap={reduce ? undefined : { scale: 0.97 }}
                className={cn(
                  "relative overflow-hidden rounded-xl border p-3 text-left transition-shadow",
                  active
                    ? "border-primary shadow-[0_0_20px_oklch(0.78_0.15_55_/_0.3)]"
                    : "border-border hover:border-foreground/25",
                )}
              >
                <span className="block h-14 rounded-lg" style={{ background: t.swatch }} />
                <span className="mt-2 flex items-center gap-1.5 text-sm font-medium">
                  {t.label}
                  {active && <Check className="size-3.5 text-primary" strokeWidth={3} />}
                </span>
                <span className="block text-[11px] text-muted-foreground">{t.desc}</span>
                {active && (
                  <motion.span
                    layoutId="theme-ring"
                    className="pointer-events-none absolute inset-0 rounded-xl ring-2 ring-primary/60 ring-inset"
                    transition={{ type: "spring", stiffness: 480, damping: 36 }}
                  />
                )}
              </motion.button>
            );
          })}
        </div>
        <label className="flex items-center gap-2 text-sm">
          <Checkbox checked={settings.reducedMotion} onCheckedChange={(v) => patch({ reducedMotion: Boolean(v) })} />
          Reduced motion
        </label>
      </section>
      <section className="space-y-3">
        <h2 className="font-medium">Backups</h2>
        <Label>Keep last</Label>
        <Input
          type="number"
          value={settings.backupRetention}
          onChange={(e) => patch({ backupRetention: Number(e.target.value) })}
        />
      </section>
      <section className="space-y-3">
        <h2 className="font-medium">Minecraft servers</h2>
        <label className="flex items-center gap-2 text-sm">
          <Checkbox checked={settings.autoUpdates} onCheckedChange={(v) => patch({ autoUpdates: Boolean(v) })} />
          Allow automatic server updates (off by default)
        </label>
      </section>
      <section className="space-y-3">
        <div className="flex items-baseline gap-2">
          <h2 className="font-medium">App updates</h2>
          <span className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground">
            v{appVersion ?? "…"}
          </span>
        </div>
        <label className="flex items-start gap-2 text-sm">
          <Checkbox
            checked={settings.appAutoUpdate}
            onCheckedChange={(v) => patch({ appAutoUpdate: Boolean(v) })}
            className="mt-0.5"
          />
          <span>
            Check for updates on launch
            <span className="block text-xs text-muted-foreground">A popup appears when a new version is ready.</span>
          </span>
        </label>
        <label className="flex items-start gap-2 text-sm">
          <Checkbox
            checked={settings.appAutoInstall}
            onCheckedChange={(v) => patch({ appAutoInstall: Boolean(v) })}
            className="mt-0.5"
          />
          <span>
            Install updates automatically
            <span className="block text-xs text-muted-foreground">Downloads and installs in the background, then restarts the app.</span>
          </span>
        </label>
        {settings.skippedAppVersion ? (
          <p className="text-xs text-muted-foreground">
            Skipping version {settings.skippedAppVersion}.{" "}
            <button className="underline underline-offset-2 hover:text-foreground" onClick={() => patch({ skippedAppVersion: null })}>
              Stop skipping
            </button>
          </p>
        ) : null}
        <Button variant="outline" disabled={updatePhase === "checking"} onClick={() => void checkNow(true)}>
          <ArrowDownToLine className="size-4" />
          {updatePhase === "checking" ? "Checking…" : "Check for updates"}
        </Button>
      </section>
      <section className="space-y-3">
        <h2 className="font-medium">Developer</h2>
        <label className="flex items-center gap-2 text-sm">
          <Checkbox checked={settings.developerMode} onCheckedChange={(v) => patch({ developerMode: Boolean(v) })} />
          Developer mode
        </label>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() =>
              void api
                .openLogs()
                .then((p) => api.openPath(p))
                .catch((e) => toast.error(errorMessage(e)))
            }
          >
            Open logs folder
          </Button>
          <Button
            variant="outline"
            onClick={() =>
              void api
                .diagnostic()
                .then((text) => navigator.clipboard.writeText(text).then(() => toast.success("Diagnostic report copied")))
                .catch((e) => toast.error(errorMessage(e)))
            }
          >
            Export diagnostic report
          </Button>
        </div>
      </section>
    </div>
  );
}
