import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { AnimatePresence, motion } from "framer-motion";
import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { api } from "@/lib/api";
import { useAppStore } from "@/stores/useAppStore";
import logo from "@/assets/logo.webp";

export function OnboardingPage() {
  const settings = useAppStore((s) => s.settings);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const javas = useAppStore((s) => s.javas);
  const [step, setStep] = useState(0);
  const [dir, setDir] = useState(settings?.defaultServersDir ?? "");
  const navigate = useNavigate();

  useEffect(() => {
    if (settings?.defaultServersDir) setDir(settings.defaultServersDir);
  }, [settings]);

  const finish = async () => {
    if (!settings) return;
    await saveSettings({
      ...settings,
      defaultServersDir: dir,
      preferredJavaPath: javas[0]?.path ?? settings.preferredJavaPath,
      onboarded: true,
    });
    navigate("/create");
  };

  return (
    <div className="relative flex h-full items-center justify-center overflow-hidden">
      <div className="pointer-events-none absolute inset-0 sf-ember-field" />
      <motion.div
        initial={{ opacity: 0, y: 16, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        className="relative w-full max-w-lg rounded-2xl border border-border/80 bg-card/90 p-8 shadow-2xl backdrop-blur-xl"
      >
        <AnimatePresence mode="wait">
          {step === 0 ? (
            <motion.div key="welcome" initial={{ opacity: 0, x: 16 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -16 }} className="space-y-5">
              <img src={logo} alt="ServerForge" width={56} height={56} className="size-14 rounded-2xl shadow-[0_0_32px_oklch(0.78_0.15_55_/_0.35)]" />
              <div>
                <h1 className="text-2xl font-semibold tracking-tight">Welcome to ServerForge</h1>
                <p className="mt-2 text-sm text-muted-foreground">
                  Create and manage Minecraft Java servers without touching a command line.
                </p>
              </div>
              <Button onClick={() => setStep(1)}>Get started</Button>
            </motion.div>
          ) : null}
          {step === 1 ? (
            <motion.div key="dir" initial={{ opacity: 0, x: 16 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -16 }} className="space-y-5">
              <h2 className="text-xl font-semibold">Default server directory</h2>
              <p className="text-sm text-muted-foreground">
                New servers will be created here unless you pick another location.
              </p>
              <div className="space-y-2">
                <Label htmlFor="dir">Location</Label>
                <div className="flex gap-2">
                  <Input id="dir" value={dir} onChange={(e) => setDir(e.target.value)} />
                  <Button
                    variant="outline"
                    onClick={async () => {
                      const selected = await open({ directory: true });
                      if (typeof selected === "string") setDir(selected);
                    }}
                  >
                    <FolderOpen className="size-4" />
                  </Button>
                </div>
              </div>
              <div className="flex gap-2">
                <Button variant="ghost" onClick={() => setStep(0)}>
                  Back
                </Button>
                <Button onClick={() => setStep(2)}>Continue</Button>
              </div>
            </motion.div>
          ) : null}
          {step === 2 ? (
            <motion.div key="java" initial={{ opacity: 0, x: 16 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -16 }} className="space-y-5">
              <h2 className="text-xl font-semibold">Java runtime</h2>
              {javas.length ? (
                <p className="text-sm text-muted-foreground">
                  Detected {javas[0].vendor} {javas[0].version} at {javas[0].path}
                </p>
              ) : (
                <p className="text-sm text-muted-foreground">
                  No Java installation was found. You can still continue — ServerForge will help you pick one when creating a server.
                </p>
              )}
              <Button
                variant="outline"
                onClick={async () => {
                  const path = await api.openLogs();
                  await api.openPath(path);
                }}
              >
                Later you can open logs from Settings
              </Button>
              <div className="flex gap-2">
                <Button variant="ghost" onClick={() => setStep(1)}>
                  Back
                </Button>
                <Button onClick={() => void finish()}>Create your first server</Button>
              </div>
            </motion.div>
          ) : null}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}
