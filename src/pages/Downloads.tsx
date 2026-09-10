import { PageHeader } from "@/components/layout/PageHeader";
import { Progress } from "@/components/ui/progress";
import { formatBytes, formatSpeed } from "@/lib/format";
import { useAppStore } from "@/stores/useAppStore";

export function DownloadsPage() {
  const downloads = useAppStore((s) => s.downloads);
  return (
    <div className="mx-auto max-w-3xl space-y-4 p-8">
      <PageHeader title="Downloads" subtitle="Live artifact and installer transfers." />
      {downloads.length === 0 ? (
        <p className="text-sm text-muted-foreground">No downloads yet.</p>
      ) : (
        downloads.map((d) => {
          const pct = d.bytesTotal ? Math.round((d.bytesDownloaded / d.bytesTotal) * 100) : 0;
          return (
            <div key={d.id} className="sf-card-hover space-y-2 rounded-xl border bg-card/70 p-4">
              <div className="flex justify-between text-sm">
                <span>{d.label}</span>
                <span className="text-muted-foreground">{d.status}</span>
              </div>
              <Progress value={pct} />
              <div className="text-xs text-muted-foreground">
                {formatBytes(d.bytesDownloaded)}
                {d.bytesTotal ? ` / ${formatBytes(d.bytesTotal)}` : ""} · {formatSpeed(d.speedBps)}
                {d.etaSeconds != null ? ` · ETA ${d.etaSeconds}s` : ""}
              </div>
            </div>
          );
        })
      )}
    </div>
  );
}
