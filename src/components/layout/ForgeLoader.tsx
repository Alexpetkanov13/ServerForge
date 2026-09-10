import { motion, useReducedMotion } from "framer-motion";
import logo from "@/assets/logo.webp";

export function ForgeLoader({ label = "Forging the control plane…" }: { label?: string }) {
  const reduce = useReducedMotion();
  return (
    <div className="relative flex h-full items-center justify-center overflow-hidden bg-background">
      <div className="pointer-events-none absolute inset-0 sf-ember-field" />
      <div className="relative flex flex-col items-center gap-4">
        <motion.div
          className="relative"
          animate={reduce ? undefined : { scale: [1, 1.04, 1] }}
          transition={{ duration: 1.8, repeat: Infinity, ease: "easeInOut" }}
        >
          <div className="absolute -inset-6 rounded-full bg-primary/20 blur-2xl" />
          <img src={logo} alt="" width={72} height={72} className="relative size-[72px] rounded-2xl shadow-[0_0_40px_oklch(0.78_0.15_55_/_0.45)]" />
        </motion.div>
        <div className="text-sm font-medium tracking-wide text-foreground">ServerForge</div>
        <p className="text-xs text-muted-foreground">{label}</p>
        <div className="h-1 w-40 overflow-hidden rounded-full bg-muted">
          <motion.div
            className="h-full w-1/2 rounded-full bg-primary"
            animate={reduce ? undefined : { x: ["-80%", "180%"] }}
            transition={{ duration: 1.2, repeat: Infinity, ease: "easeInOut" }}
          />
        </div>
      </div>
    </div>
  );
}
