import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import { api, errorMessage } from "@/lib/api";
import { useServerStore } from "@/stores/useServerStore";
import type { ServerRecord } from "@/types";

export async function pickAndImportSharePack(): Promise<ServerRecord | null> {
  const selected = await open({
    title: "Import ServerForge server",
    multiple: false,
    filters: [{ name: "ServerForge server", extensions: ["server"] }],
  });
  if (typeof selected !== "string") return null;
  try {
    const server = await api.importSharePack(selected);
    await useServerStore.getState().refresh();
    toast.success(`Imported ${server.name}.`);
    return server;
  } catch (err) {
    toast.error(errorMessage(err));
    return null;
  }
}
