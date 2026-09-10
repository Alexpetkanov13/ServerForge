import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { ArrowRight, Download, LoaderCircle, Rocket, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { formatBytes } from "@/lib/format";
import { useUpdateStore } from "@/stores/useUpdateStore";
import { cn } from "@/lib/utils";

function percent(downloaded: number, total: number | null) {
  if (!total || total <= 0) return null;
  return Math.min(100, Math.max(0, (downloaded / total) * 100));
}

export function UpdateDialog() {
  const reduce = useReducedMotion();
  const phase = useUpdateStore((s) => s.phase);
  const meta = useUpdateStore((s) => s.meta);
  const downloaded = useUpdateStore((s) => s.downloaded);
  const total = useUpdateStore((s) => s.total);
  const error = useUpdateStore((s) => s.error);
  const dismissed = useUpdateStore((s) => s.dismissed);
  const startInstall = useUpdateStore((s) => s.startInstall);
  const dismiss = useUpdateStore((s) => s.dismiss);
  const skipVersion = useUpdateStore((s) => s.skipVersion);

  const open = !dismissed && meta && (phase === "available" || phase === "downloading" || phase === "installing" || phase === "error");
  const pct = percent(downloaded, total);
  const busy = phase === "downloading" || phase === "installing";

  return (
    <AnimatePresence>
      {open ? (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-[100] grid place-items-center bg-black/60 p-4 backdrop-blur-sm"
          onClick={() => {
            if (!busy) dismiss();
          }}
        >
          <motion.div
            role="dialog"
            aria-modal="true"
            aria-label="Application update"
            initial={reduce ? { opacity: 0 } : { opacity: 0, y: 28, scale: 0.94 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={reduce ? { opacity: 0 } : { opacity: 0, y: 16, scale: 0.96 }}
            transition={{ type: "spring", stiffness: 380, damping: 30 }}
            onClick={(e) => e.stopPropagation()}
            className="relative w-full max-w-md overflow-hidden rounded-2xl border border-border bg-card shadow-[0_40px_120px_-20px_oklch(0_0_0_/_0.8)]"
          >
            <div className="h-1 bg-gradient-to-r from-primary via-amber-400 to-emerald-400" />
            <div className="space-y-4 p-5">
              <div className="flex items-start gap-3">
                <motion.span
                  animate={reduce || busy ? undefined : { rotate: [0, -8, 8, 0] }}
                  transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
                  className="grid size-11 shrink-0 place-items-center rounded-2xl bg-primary/10 text-primary shadow-[0_0_24px_oklch(0.78_0.15_55_/_0.35)]"
                >
                  {busy ? <LoaderCircle className="size-5 animate-spin" /> : <Rocket className="size-5" />}
                </motion.span>
                <div className="min-w-0">
                  <h2 className="text-base font-semibold tracking-tight">
                    {phase === "installing" ? "Installing update…" : phase === "downloading" ? "Downloading update…" : phase === "error" ? "Update failed" : "Update available"}
                  </h2>
                  <div className="mt-1.5 flex flex-wrap items-center gap-1.5 font-mono text-[11px]">
                    <span className="rounded-md bg-muted px-1.5 py-0.5 text-muted-foreground">v{meta.currentVersion}</span>
                    <ArrowRight className="size-3 text-muted-foreground" />
                    <span className="rounded-md bg-primary/15 px-1.5 py-0.5 font-semibold text-primary">v{meta.version}</span>
                    {meta.date ? <span className="text-muted-foreground">{new Date(meta.date).toLocaleDateString()}</span> : null}
                  </div>
                </div>
                {!busy && (
                  <button onClick={() => dismiss()} aria-label="Close" className="ml-auto rounded-lg p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground">
                    <X className="size-4" />
                  </button>
                )}
              </div>

              {phase === "error" ? (
                <p className="rounded-xl border border-destructive/40 bg-destructive/[0.07] p-3 text-sm">{error ?? "Something went wrong."}</p>
              ) : busy ? (
                <div className="space-y-2">
                  <Progress value={pct ?? (phase === "installing" ? 100 : 8)} className="h-2.5" />
                  <p className="text-xs text-muted-foreground">
                    {phase === "installing"
                      ? "Handing off to the installer — ServerForge will restart automatically."
                      : pct !== null
                        ? `${formatBytes(downloaded)} of ${formatBytes(total ?? 0)} (${pct.toFixed(0)}%)`
                        : `${formatBytes(downloaded)} downloaded…`}
                  </p>
                </div>
              ) : (
                meta.body && (
                  <div className="max-h-44 overflow-auto rounded-xl border border-border/60 bg-muted/30 p-3 text-[13px] leading-relaxed text-muted-foreground">
                    <div className="mb-1 text-[11px] font-semibold uppercase tracking-wider text-foreground/70">What's new</div>
                    <p className="whitespace-pre-wrap">{meta.body}</p>
                  </div>
                )
              )}

              <div className={cn("flex items-center gap-2", phase === "available" && "justify-between")}>
                {phase === "available" && (
                  <Button variant="ghost" size="sm" onClick={() => void skipVersion()} className="text-muted-foreground">
                    Skip this version
                  </Button>
                )}
                <div className="ml-auto flex gap-2">
                  {phase === "error" ? (
                    <>
                      <Button variant="outline" onClick={() => dismiss()}>Close</Button>
                      <Button onClick={() => void startInstall()}>Try again</Button>
                    </>
                  ) : (
                    !busy && (
                      <>
                        <Button variant="outline" onClick={() => dismiss()}>Later</Button>
                        <motion.div whileTap={reduce ? undefined : { scale: 0.96 }}>
                          <Button onClick={() => void startInstall()} className="shadow-[0_0_20px_oklch(0.78_0.15_55_/_0.35)]">
                            <Download className="size-4" /> Download & install
                          </Button>
                        </motion.div>
                      </>
                    )
                  )}
                </div>
              </div>
              {!busy && phase !== "error" && (
                <p className="text-center text-[11px] text-muted-foreground">Installing restarts ServerForge automatically.</p>
              )}
            </div>
          </motion.div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
