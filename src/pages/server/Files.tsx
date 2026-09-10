import { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { toast } from "sonner";
import {
  ChevronRight,
  FilePlus,
  FileUp,
  FolderPlus,
  FolderUp,
  File as FileIcon,
  Folder,
  Save,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { api, errorMessage } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import type { FileEntry } from "@/types";

const EDITABLE = /\.(txt|properties|yml|yaml|json|log|toml|cfg|conf|md|xml|csv|ini|nfo|sh|bat|ps1|js|ts|toml)$/i;

function joinRel(rel: string, name: string) {
  if (!rel || rel === ".") return name;
  return `${rel.replace(/\\/g, "/")}/${name}`;
}

export function FilesPage() {
  const { id } = useParams();
  const [rel, setRel] = useState(".");
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [editing, setEditing] = useState<string | null>(null);
  const [contents, setContents] = useState("");
  const [dirty, setDirty] = useState(false);
  const [newName, setNewName] = useState("");
  const [creating, setCreating] = useState<"file" | "folder" | null>(null);
  const [busy, setBusy] = useState(false);

  const crumbs = useMemo(() => {
    if (!rel || rel === ".") return [] as string[];
    return rel.replace(/\\/g, "/").split("/").filter(Boolean);
  }, [rel]);

  const load = (path: string) => {
    if (!id) return;
    const next = path || ".";
    setRel(next);
    void api
      .files(id, next === "." ? "" : next)
      .then(setEntries)
      .catch((e) => toast.error(errorMessage(e)));
  };

  useEffect(() => {
    load(".");
  }, [id]);

  if (!id) return null;

  const createItem = async () => {
    const name = newName.trim();
    if (!name || !creating) return;
    try {
      const path = joinRel(rel, name);
      if (creating === "folder") {
        await api.createFolder(id, path);
      } else {
        await api.writeFile(id, path, "");
        setEditing(path);
        setContents("");
        setDirty(false);
      }
      setNewName("");
      setCreating(null);
      load(rel);
      toast.success(creating === "folder" ? "Folder created." : "File created.");
    } catch (err) {
      toast.error(errorMessage(err));
    }
  };

  const upload = async (directory: boolean) => {
    const selected = await open({
      multiple: !directory,
      directory,
      title: directory ? "Upload folder" : "Upload files",
    });
    if (!selected) return;
    const sources = Array.isArray(selected) ? selected : [selected];
    setBusy(true);
    try {
      const count = await api.uploadInto(id, rel === "." ? "" : rel, sources);
      toast.success(`Uploaded ${count} item${count === 1 ? "" : "s"}.`);
      load(rel);
    } catch (err) {
      toast.error(errorMessage(err));
    } finally {
      setBusy(false);
    }
  };

  const openEntry = async (entry: FileEntry) => {
    const next = joinRel(rel, entry.name);
    if (entry.isDir) {
      load(next);
      return;
    }
    if (!EDITABLE.test(entry.name)) {
      toast.message("This file type can’t be edited here. Upload a replacement or open the folder.");
      return;
    }
    try {
      const text = await api.readFile(id, next);
      setEditing(next);
      setContents(text);
      setDirty(false);
    } catch (err) {
      toast.error(errorMessage(err));
    }
  };

  const save = async () => {
    if (!editing) return;
    try {
      await api.writeFile(id, editing, contents);
      setDirty(false);
      toast.success("Saved.");
    } catch (err) {
      toast.error(errorMessage(err));
    }
  };

  const remove = async (entry: FileEntry) => {
    const path = joinRel(rel, entry.name);
    try {
      await api.deleteFile(id, path);
      if (editing === path) {
        setEditing(null);
        setContents("");
      }
      load(rel);
    } catch (err) {
      toast.error(errorMessage(err));
    }
  };

  return (
    <div className="grid min-h-[520px] gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)]">
      <div className="flex min-h-0 flex-col rounded-2xl border border-border/80 bg-card/60">
        <div className="flex flex-wrap items-center gap-2 border-b border-border/70 px-4 py-3">
          <button className="text-xs text-muted-foreground hover:text-foreground" onClick={() => load(".")}>
            Root
          </button>
          {crumbs.map((part, i) => {
            const path = crumbs.slice(0, i + 1).join("/");
            return (
              <span key={path} className="flex items-center gap-1 text-xs">
                <ChevronRight className="size-3 text-muted-foreground" />
                <button className="hover:text-foreground" onClick={() => load(path)}>
                  {part}
                </button>
              </span>
            );
          })}
        </div>
        <div className="flex flex-wrap gap-2 border-b border-border/70 px-4 py-3">
          <Button size="sm" variant="outline" onClick={() => setCreating("file")}>
            <FilePlus className="size-3.5" /> New file
          </Button>
          <Button size="sm" variant="outline" onClick={() => setCreating("folder")}>
            <FolderPlus className="size-3.5" /> New folder
          </Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => void upload(false)}>
            <FileUp className="size-3.5" /> Upload file
          </Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => void upload(true)}>
            <FolderUp className="size-3.5" /> Upload folder
          </Button>
        </div>
        {creating ? (
          <div className="flex gap-2 border-b border-border/70 px-4 py-3">
            <Input
              autoFocus
              placeholder={creating === "folder" ? "plugins/custom" : "notes.txt"}
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void createItem();
                if (e.key === "Escape") setCreating(null);
              }}
            />
            <Button size="sm" onClick={() => void createItem()}>
              Create
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setCreating(null)}>
              Cancel
            </Button>
          </div>
        ) : null}
        <div className="min-h-0 flex-1 overflow-auto p-2">
          {entries.length === 0 ? (
            <p className="px-3 py-8 text-center text-sm text-muted-foreground">This folder is empty.</p>
          ) : (
            entries.map((entry) => (
              <div
                key={entry.path}
                className="group flex items-center gap-2 rounded-xl px-2 py-1.5 hover:bg-muted/60"
              >
                <button className="flex min-w-0 flex-1 items-center gap-3 text-left text-sm" onClick={() => void openEntry(entry)}>
                  {entry.isDir ? (
                    <Folder className="size-4 shrink-0 text-amber-400" />
                  ) : (
                    <FileIcon className="size-4 shrink-0 text-muted-foreground" />
                  )}
                  <span className="truncate">{entry.name}</span>
                  <span className="ml-auto text-xs text-muted-foreground">
                    {entry.isDir ? "" : formatBytes(entry.size)}
                  </span>
                </button>
                <Button
                  size="icon-xs"
                  variant="ghost"
                  className="opacity-0 group-hover:opacity-100"
                  onClick={() => void remove(entry)}
                >
                  <Trash2 className="size-3.5" />
                </Button>
              </div>
            ))
          )}
        </div>
      </div>
      <div className="flex min-h-0 flex-col rounded-2xl border border-border/80 bg-card/60 p-4">
        {editing ? (
          <>
            <div className="mb-3 flex items-center justify-between gap-3">
              <div>
                <div className="text-sm font-medium">Edit file</div>
                <div className="font-mono text-xs text-muted-foreground">{editing}</div>
              </div>
              <Button onClick={() => void save()} disabled={!dirty}>
                <Save className="size-3.5" /> Save
              </Button>
            </div>
            <Textarea
              className="min-h-0 flex-1 font-mono text-xs"
              value={contents}
              onChange={(e) => {
                setContents(e.target.value);
                setDirty(true);
              }}
            />
          </>
        ) : (
          <div className="flex flex-1 flex-col items-center justify-center text-center text-sm text-muted-foreground">
            <FileIcon className="mb-3 size-8 opacity-50" />
            <p>Create a file or open a text config to edit it here.</p>
          </div>
        )}
      </div>
    </div>
  );
}
