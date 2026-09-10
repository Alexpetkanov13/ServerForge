import { useParams } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { api, errorMessage } from "@/lib/api";
import { useServerStore } from "@/stores/useServerStore";

export function UpdatesPage() {
  const { id } = useParams();
  const server = useServerStore((s) => s.servers.find((x) => x.id === id));
  if (!id || !server) return null;
  return (
    <div className="max-w-lg space-y-3 text-sm">
      <p>
        Current: {server.provider} {server.minecraftVersion} {server.build ? `build ${server.build}` : ""}
      </p>
      <p className="text-muted-foreground">
        Updates never run automatically unless you enable them in Settings. A backup is created before replacing server software.
      </p>
      <Button
        onClick={() =>
          void api
            .createBackup(id, "pre_update")
            .then(() => toast.success("Backup created. Choose a new version from Create-like update flow next."))
            .catch((e) => toast.error(errorMessage(e)))
        }
      >
        Backup before updating
      </Button>
    </div>
  );
}

export function PlayersPage() {
  return (
    <p className="text-sm text-muted-foreground">
      Player counts appear on the overview while the server is online. Use the console for whitelist, op, and kick commands.
    </p>
  );
}
