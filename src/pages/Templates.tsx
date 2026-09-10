import { useNavigate } from "react-router-dom";
import { motion } from "framer-motion";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PageHeader } from "@/components/layout/PageHeader";
import { itemVariants, listVariants } from "@/lib/motion";
import { useWizardStore } from "@/stores/useWizardStore";

const templates = [
  { name: "Paper Survival", provider: "paper", ram: 4096, desc: "High-performance plugin server." },
  { name: "Vanilla Survival", provider: "vanilla", ram: 3072, desc: "Official Mojang server jar." },
  { name: "Fabric Modded", provider: "fabric", ram: 6144, desc: "Lightweight mod loader." },
  { name: "Paper SMP", provider: "paper", ram: 8192, desc: "Larger community SMP defaults." },
];

export function TemplatesPage() {
  const patch = useWizardStore((s) => s.patch);
  const setStep = useWizardStore((s) => s.setStep);
  const navigate = useNavigate();
  return (
    <div className="mx-auto max-w-4xl space-y-6 p-8">
      <PageHeader title="Templates" subtitle="Start from a tuned preset, then finish the create wizard." />
      <motion.div className="grid gap-3 sm:grid-cols-2" variants={listVariants} initial="hidden" animate="show">
        {templates.map((t) => (
          <motion.button
            key={t.name}
            variants={itemVariants}
            whileHover={{ y: -3 }}
            whileTap={{ scale: 0.98 }}
            className="text-left"
            onClick={() => {
              patch({
                name: t.name,
                provider: t.provider,
                memoryMaxMb: t.ram,
                memoryMinMb: Math.floor(t.ram / 2),
              });
              setStep(0);
              navigate("/create");
            }}
          >
            <Card className="sf-card-hover h-full bg-card/80">
              <CardHeader>
                <CardTitle className="text-base">{t.name}</CardTitle>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground">{t.desc}</CardContent>
            </Card>
          </motion.button>
        ))}
      </motion.div>
    </div>
  );
}
