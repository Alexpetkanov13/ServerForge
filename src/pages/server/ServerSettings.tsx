import { memo, useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { toast } from "sonner";
import { AlertTriangle, Globe, RotateCcw, Save, SlidersHorizontal, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api, errorMessage } from "@/lib/api";
import type { PropertyField } from "@/types";
import { useServerStore } from "@/stores/useServerStore";
import { DeleteServerButton } from "@/components/server/DeleteServerButton";
import { cn } from "@/lib/utils";

const SECTIONS = [
  { id: "players", title: "Players & access", icon: Users, keys: ["max-players", "online-mode", "allow-flight", "pvp"] },
  { id: "world", title: "World & gameplay", icon: Globe, keys: ["difficulty", "gamemode", "view-distance", "simulation-distance"] },
  { id: "identity", title: "Identity", icon: SlidersHorizontal, keys: ["motd"] },
] as const;

const FieldControl = memo(function FieldControl({
  field,
  onChange,
}: {
  field: PropertyField;
  onChange: (key: string, value: string) => void;
}) {
  const highlight = field.key === "max-players";
  return (
    <motion.div
      layout
      className={cn(
        "relative space-y-1.5 rounded-xl border p-3 transition-colors",
        highlight ? "border-primary/50 bg-primary/[0.05] shadow-[0_0_20px_oklch(0.78_0.15_55_/_0.15)]" : "border-border/60 bg-muted/20",
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <Label className="text-[13px]">{field.label}</Label>
        {highlight && (
          <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-primary">
            Live in Overview
          </span>
        )}
      </div>
      {field.kind === "bool" ? (
        <label className="flex cursor-pointer items-center gap-2 text-sm text-muted-foreground">
          <Checkbox checked={field.value === "true"} onCheckedChange={(v) => onChange(field.key, String(Boolean(v)))} />
          {field.value === "true" ? "Enabled" : "Disabled"}
        </label>
      ) : field.options.length ? (
        <select
          className="h-9 w-full rounded-lg border border-border bg-background px-2 text-sm outline-none transition-shadow focus-visible:shadow-[0_0_16px_oklch(0.78_0.15_55_/_0.25)]"
          value={field.value}
          onChange={(e) => onChange(field.key, e.target.value)}
        >
          {field.options.map((o) => (
            <option key={o}>{o}</option>
          ))}
        </select>
      ) : (
        <Input
          value={field.value}
          type={field.key === "max-players" || field.key.includes("distance") ? "number" : "text"}
          min={field.key === "max-players" ? 1 : undefined}
          onChange={(e) => onChange(field.key, e.target.value)}
          className={cn(highlight && "font-mono")}
        />
      )}
      {highlight && <p className="text-[11px] text-muted-foreground">Changing this updates the Overview player count instantly.</p>}
    </motion.div>
  );
});

export function ServerSettingsPage() {
  const { id } = useParams();
  const reduce = useReducedMotion();
  const server = useServerStore((s) => s.servers.find((x) => x.id === id));
  const refresh = useServerStore((s) => s.refresh);
  const [fields, setFields] = useState<PropertyField[]>([]);
  const [saving, setSaving] = useState(false);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    if (!id) return;
    setLoaded(false);
    void api
      .properties(id)
      .then((p) => {
        setFields(p.fields);
        setLoaded(true);
      })
      .catch((e) => toast.error(errorMessage(e)));
  }, [id]);

  const byKey = useMemo(() => Object.fromEntries(fields.map((f) => [f.key, f])), [fields]);
  const online = server?.status === "online" || server?.status === "starting";

  if (!id || !server) return null;

  const patch = (key: string, value: string) =>
    setFields((all) => all.map((f) => (f.key === key ? { ...f, value } : f)));

  const save = () => {
    const updates = Object.fromEntries(fields.map((f) => [f.key, f.value]));
    setSaving(true);
    void api
      .saveProperties(id, updates)
      .then(() => refresh())
      .then(() => {
        toast.success(online ? "Settings saved — restart the server to apply them in-game" : "Settings saved");
      })
      .catch((e) => toast.error(errorMessage(e)))
      .finally(() => setSaving(false));
  };

  return (
    <motion.div
      initial={reduce ? false : { opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
      className="max-w-2xl space-y-5"
    >
      <AnimatePresence>
        {online && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            className="overflow-hidden"
          >
            <div className="flex items-start gap-2.5 rounded-xl border border-amber-500/30 bg-amber-500/[0.07] p-3 text-sm">
              <AlertTriangle className="mt-0.5 size-4 shrink-0 text-amber-400" />
              <p className="text-muted-foreground">
                This server is <span className="font-medium text-foreground">running</span>. Saved values update the
                Overview instantly, but the game server applies most of them after a{" "}
                <button
                  className="inline-flex items-center gap-1 font-medium text-foreground underline decoration-amber-400/60 underline-offset-2"
                  onClick={() =>
                    void api
                      .restart(id)
                      .then(() => refresh().then(() => toast.success("Server restarted")))
                      .catch((e) => toast.error(errorMessage(e)))
                  }
                >
                  <RotateCcw className="size-3" /> restart
                </button>
                .
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {!loaded ? (
        <div className="grid gap-3 sm:grid-cols-2">
          {[0, 1, 2, 3].map((i) => (
            <motion.div
              key={i}
              animate={reduce ? undefined : { opacity: [0.4, 0.8, 0.4] }}
              transition={{ duration: 1.4, repeat: Infinity, delay: i * 0.15 }}
              className="h-24 rounded-xl border border-border bg-muted/40"
            />
          ))}
        </div>
      ) : (
        SECTIONS.map((section, si) => {
          const items = section.keys
            .map((k) => byKey[k])
            .filter((f): f is PropertyField => Boolean(f));
          if (!items.length) return null;
          return (
            <motion.div
              key={section.id}
              initial={reduce ? false : { opacity: 0, y: 14 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.32, delay: si * 0.07, ease: [0.22, 1, 0.36, 1] }}
            >
              <Card className="overflow-hidden">
                <div className="flex items-center gap-2 border-b border-border/60 bg-muted/30 px-4 py-2.5">
                  <section.icon className="size-4 text-primary" />
                  <h2 className="text-sm font-semibold">{section.title}</h2>
                </div>
                <CardContent className="grid gap-3 p-4 sm:grid-cols-2">
                  {items.map((field) => (
                    <FieldControl key={field.key} field={field} onChange={patch} />
                  ))}
                </CardContent>
              </Card>
            </motion.div>
          );
        })
      )}

      <div className="flex items-center gap-2">
        <motion.div whileTap={reduce ? undefined : { scale: 0.96 }} whileHover={reduce ? undefined : { scale: 1.02 }}>
          <Button onClick={save} disabled={saving || !loaded} className="shadow-[0_0_20px_oklch(0.78_0.15_55_/_0.3)]">
            <Save className="size-4" /> {saving ? "Saving…" : "Save settings"}
          </Button>
        </motion.div>
      </div>

      <motion.div
        initial={reduce ? false : { opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.25 }}
        className="space-y-2 rounded-xl border border-destructive/30 bg-destructive/[0.04] p-4"
      >
        <h2 className="text-sm font-medium">Danger zone</h2>
        <p className="text-sm text-muted-foreground">
          Stop this server and remove it from ServerForge. You can also delete its folder from disk.
        </p>
        <DeleteServerButton id={id} name={server.name} />
      </motion.div>
    </motion.div>
  );
}
