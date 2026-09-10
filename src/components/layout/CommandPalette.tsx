import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { pickAndImportSharePack } from "@/lib/sharePack";
import { useServerStore } from "@/stores/useServerStore";

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const servers = useServerStore((s) => s.servers);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const go = (path: string) => {
    setOpen(false);
    navigate(path);
  };

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <Command>
        <CommandInput placeholder="Search servers and commands…" />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Commands">
            <CommandItem onSelect={() => go("/create")}>Create Server</CommandItem>
            <CommandItem
              onSelect={() => {
                setOpen(false);
                void pickAndImportSharePack().then((server) => {
                  if (server) navigate(`/servers/${server.id}`);
                });
              }}
            >
              Import .server file
            </CommandItem>
            <CommandItem onSelect={() => go("/servers")}>Open Servers</CommandItem>
            <CommandItem onSelect={() => go("/java")}>Manage Java</CommandItem>
            <CommandItem onSelect={() => go("/settings")}>Open Settings</CommandItem>
          </CommandGroup>
          <CommandGroup heading="Servers">
            {servers.map((server) => (
              <CommandItem key={server.id} onSelect={() => go(`/servers/${server.id}`)}>
                {server.name}
              </CommandItem>
            ))}
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
