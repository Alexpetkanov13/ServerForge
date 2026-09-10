import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { api, errorMessage } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import type { WorldInfo } from "@/types";

export function WorldsPage() {
  const { id } = useParams();
  const [worlds, setWorlds] = useState<WorldInfo[]>([]);
  useEffect(() => {
    if (id) void api.worlds(id).then(setWorlds);
  }, [id]);
  return (
    <div className="space-y-3">
      {worlds.map((world) => (
        <div key={world.path} className="flex items-center justify-between rounded-lg border p-4">
          <div>
            <div className="font-medium">{world.name}</div>
            <div className="text-sm text-muted-foreground">
              {world.kind} · {formatBytes(world.sizeBytes)}
            </div>
          </div>
          <Button variant="outline" onClick={() => void api.openPath(world.path).catch((e) => toast.error(errorMessage(e)))}>
            Open folder
          </Button>
        </div>
      ))}
    </div>
  );
}
