import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { api, errorMessage } from "@/lib/api";
import { PageHeader } from "@/components/layout/PageHeader";
import { useAppStore } from "@/stores/useAppStore";

export function JavaPage() {
  const javas = useAppStore((s) => s.javas);
  const load = useAppStore((s) => s.load);
  return (
    <div className="mx-auto max-w-4xl space-y-6 p-8">
      <PageHeader title="Java" subtitle="Detected runtimes are never modified silently.">
        <Button variant="outline" onClick={() => void load()}>
          Rescan
        </Button>
      </PageHeader>
      {javas.length === 0 ? (
        <Card>
          <CardContent className="space-y-3 p-6 text-sm">
            <p>No Java installation was detected.</p>
            <Button
              onClick={() =>
                void api
                  .javaDownloadUrl(21)
                  .then((url) => window.open(url, "_blank"))
                  .catch((e) => toast.error(errorMessage(e)))
              }
            >
              Download Java 21 (Eclipse Temurin)
            </Button>
          </CardContent>
        </Card>
      ) : (
        javas.map((j) => (
          <Card key={j.path} className="sf-card-hover bg-card/80">
            <CardContent className="flex items-center justify-between p-5">
              <div>
                <div className="font-medium">
                  Java {j.major} · {j.vendor} · {j.architecture}
                </div>
                <div className="text-sm text-muted-foreground">{j.path}</div>
                <div className="text-xs text-muted-foreground">{j.version}</div>
              </div>
              <Button
                variant="outline"
                onClick={() =>
                  void api
                    .testJava(j.path)
                    .then(() => toast.success("Java is working"))
                    .catch((e) => toast.error(errorMessage(e)))
                }
              >
                Test
              </Button>
            </CardContent>
          </Card>
        ))
      )}
    </div>
  );
}
