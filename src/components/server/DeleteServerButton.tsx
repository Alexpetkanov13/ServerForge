import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { Trash2 } from "lucide-react";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { api, errorMessage } from "@/lib/api";
import { useServerStore } from "@/stores/useServerStore";

export function DeleteServerButton({
  id,
  name,
  variant = "destructive",
}: {
  id: string;
  name: string;
  variant?: "destructive" | "outline" | "ghost";
}) {
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [deleteFiles, setDeleteFiles] = useState(true);
  const refresh = useServerStore((s) => s.refresh);
  const navigate = useNavigate();

  const remove = async () => {
    setBusy(true);
    try {
      await api.deleteServer(id, deleteFiles);
      await refresh();
      toast.success(deleteFiles ? `${name} and its files were deleted.` : `${name} was removed from ServerForge.`);
      setOpen(false);
      navigate("/servers");
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <AlertDialog
      open={open}
      onOpenChange={(next) => {
        if (busy && !next) return;
        setOpen(next);
      }}
    >
      <AlertDialogTrigger asChild>
        <Button variant={variant}>
          <Trash2 className="size-4" />
          Delete server
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent className="sm:max-w-md">
        <AlertDialogHeader>
          <AlertDialogTitle>Delete {name}?</AlertDialogTitle>
          <AlertDialogDescription>
            The server will be stopped first. This cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <label className="flex items-start gap-2 text-sm">
          <Checkbox checked={deleteFiles} onCheckedChange={(v) => setDeleteFiles(Boolean(v))} className="mt-0.5" />
          <span>
            Also delete the server folder from disk, including worlds, plugins, mods, and configs.
          </span>
        </label>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={busy}>Cancel</AlertDialogCancel>
          <Button variant="destructive" disabled={busy} onClick={() => void remove()}>
            {busy ? "Deleting…" : deleteFiles ? "Delete everything" : "Remove from ServerForge"}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
