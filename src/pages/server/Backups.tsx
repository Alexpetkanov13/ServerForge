import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { api, errorMessage } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import type { BackupRecord } from "@/types";

export function BackupsPage() {
  const { id } = useParams();
  const [items, setItems] = useState<BackupRecord[]>([]);
  const reload = () => {
    if (id) void api.backups(id).then(setItems);
  };
  useEffect(reload, [id]);
  if (!id) return null;
  return (
    <div className="space-y-4">
      <Button
        onClick={() =>
          void api
            .createBackup(id)
            .then(() => {
              toast.success("Backup created");
              reload();
            })
            .catch((e) => toast.error(errorMessage(e)))
        }
      >
        Create backup
      </Button>
      {items.map((item) => (
        <div key={item.id} className="flex items-center justify-between rounded-lg border p-4 text-sm">
          <div>
            <div className="font-medium">{new Date(item.createdAt).toLocaleString()}</div>
            <div className="text-muted-foreground">
              {formatBytes(item.sizeBytes)} · {item.trigger}
            </div>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => void api.restoreBackup(id, item.id).then(() => toast.success("Restored"))}>
              Restore
            </Button>
            <Button variant="destructive" onClick={() => void api.deleteBackup(item.id).then(reload)}>
              Delete
            </Button>
          </div>
        </div>
      ))}
    </div>
  );
}
