import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { toast } from "sonner";
import {
  ArrowDownToLine,
  BrushCleaning,
  ChevronRight,
  Copy,
  Download,
  Pause,
  Play,
  Search,
  TerminalSquare,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { api, errorMessage } from "@/lib/api";
import { useConsoleStore } from "@/stores/useConsoleStore";
import { useServerStore } from "@/stores/useServerStore";
import type { ServerLogLine } from "@/types";
import { cn } from "@/lib/utils";

const EMPTY_LINES: ServerLogLine[] = [];
const RENDER_CAP = 800;

function toneOf(line: ServerLogLine): string {
  if (line.stream === "stdin") return "text-amber-300";
  if (line.stream === "stderr") return "text-red-400";
  const upper = line.line.toUpperCase();
  if (upper.includes("ERROR") || upper.includes("FATAL") || upper.includes("EXCEPTION") || upper.includes("FAILED"))
    return "text-red-400";
  if (upper.includes("WARN")) return "text-amber-300";
  if (upper.includes("DONE") || upper.includes("STARTED") || upper.includes("JOINED") || upper.includes("ONLINE"))
    return "text-emerald-300/90";
  return "text-zinc-200";
}

function stamp(value: string) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleTimeString([], { hour12: false });
}

const LogRow = memo(function LogRow({
  line,
  showStamp,
  animate,
}: {
  line: ServerLogLine;
  showStamp: boolean;
  animate: boolean;
}) {
  const isEcho = line.stream === "stdin";
  const ts = showStamp ? stamp(line.timestamp) : "";
  return (
    <div
      // Only animate fresh rows at the tail — animating hundreds of historic lines would jank.
      className={cn(
        "whitespace-pre-wrap break-words rounded px-2 py-[1px] [content-visibility:auto] [contain-intrinsic-size:auto_20px]",
        animate && (isEcho ? "sf-log-echo" : "sf-log-line"),
        toneOf(line),
        isEcho && "my-1 border-l-2 border-amber-400/70 bg-amber-400/[0.07] py-1 font-semibold",
        line.stream === "stderr" && "border-l-2 border-red-500/60 bg-red-500/[0.06] py-1",
      )}
    >
      {ts ? <span className="mr-2 select-none text-zinc-600">{ts}</span> : null}
      {line.line}
    </div>
  );
});

export function ConsolePage() {
  const { id } = useParams();
  const reduce = useReducedMotion();
  const server = useServerStore((s) => s.servers.find((x) => x.id === id));
  const lines = useConsoleStore((s) => (id ? s.lines[id] : undefined) ?? EMPTY_LINES);
  const seed = useConsoleStore((s) => s.seed);
  const echo = useConsoleStore((s) => s.echo);
  const pushError = useConsoleStore((s) => s.pushError);
  const clear = useConsoleStore((s) => s.clear);

  const [query, setQuery] = useState("");
  const [command, setCommand] = useState("");
  const [level, setLevel] = useState("all");
  const [stamps, setStamps] = useState(true);
  const [wrap, setWrap] = useState(true);
  const [live, setLive] = useState(true);
  const [sending, setSending] = useState(false);
  const [pinned, setPinned] = useState(true);
  const [histIndex, setHistIndex] = useState(-1);
  const scroller = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const history = useRef<string[]>([]);
  const online = server?.status === "online" || server?.status === "starting";

  // Seed once from latest.log — store guards against re-seeding after Clear.
  useEffect(() => {
    if (!id) return;
    try {
      void api
        .readFile(id, "logs/latest.log")
        .then((text) => seed(id, text))
        .catch(() => {});
    } catch {
      // Plain browser preview without Tauri.
    }
  }, [id, seed]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const lvl = level === "all" ? "" : level.toUpperCase();
    if (!q && !lvl) return lines;
    return lines.filter((l) => {
      if (lvl === "ERROR") {
        const u = l.line.toUpperCase();
        if (!(u.includes("ERROR") || u.includes("FATAL") || u.includes("EXCEPTION"))) return false;
      } else if (lvl && !l.line.toUpperCase().includes(lvl)) {
        return false;
      }
      if (q && !l.line.toLowerCase().includes(q)) return false;
      return true;
    });
  }, [lines, query, level]);

  const visible = useMemo(() => {
    if (filtered.length <= RENDER_CAP) return { rows: filtered, capped: 0 };
    return { rows: filtered.slice(filtered.length - RENDER_CAP), capped: filtered.length - RENDER_CAP };
  }, [filtered]);

  // Stick-to-bottom: only force scroll when the user is already pinned.
  const onScroll = useCallback(() => {
    const el = scroller.current;
    if (!el) return;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 64;
    setPinned(nearBottom);
  }, []);

  useEffect(() => {
    if (!live || !pinned) return;
    const el = scroller.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [visible.rows.length, live, pinned]);

  const jumpLatest = useCallback(() => {
    const el = scroller.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: reduce ? "auto" : "smooth" });
    setPinned(true);
  }, [reduce]);

  const send = useCallback(
    async (raw: string) => {
      if (!id) return;
      const text = raw.trim();
      if (!text || sending) return;
      // Always echo the typed command first so it + the response are visible.
      echo(id, text);
      history.current = [text, ...history.current].slice(0, 50);
      setHistIndex(-1);
      setCommand("");
      setPinned(true);
      setSending(true);
      try {
        await api.command(id, text);
      } catch (err) {
        const msg = errorMessage(err);
        pushError(id, `[console] ${msg}`);
        toast.error(msg);
      } finally {
        setSending(false);
        inputRef.current?.focus();
      }
    },
    [id, sending, echo, pushError],
  );

  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "ArrowUp") {
        e.preventDefault();
        const h = history.current;
        if (!h.length) return;
        const next = Math.min(histIndex + 1, h.length - 1);
        setHistIndex(next);
        setCommand(h[next]);
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        const h = history.current;
        if (histIndex <= 0) {
          setHistIndex(-1);
          setCommand("");
        } else {
          const next = histIndex - 1;
          setHistIndex(next);
          setCommand(h[next]);
        }
      }
    },
    [histIndex],
  );

  const copyAll = useCallback(() => {
    void navigator.clipboard
      .writeText(filtered.map((l) => `${stamp(l.timestamp) ? `[${stamp(l.timestamp)}] ` : ""}${l.line}`).join("\n"))
      .then(() => toast.success(`Copied ${filtered.length} lines`))
      .catch(() => toast.error("Copy failed"));
  }, [filtered]);

  const download = useCallback(() => {
    const blob = new Blob([filtered.map((l) => l.line).join("\n")], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${server?.name ?? "server"}-console.log`;
    a.click();
    URL.revokeObjectURL(url);
  }, [filtered, server?.name]);

  if (!id) return null;

  return (
    <motion.div
      initial={reduce ? false : { opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: [0.22, 1, 0.36, 1] }}
      className="flex h-full min-h-0 flex-col gap-3"
    >
      {/* Toolbar */}
      <div className="flex shrink-0 flex-wrap items-center gap-2">
        <div className="relative">
          <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search logs…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-52 pl-8 transition-shadow focus-visible:shadow-[0_0_0_1px_var(--ring),0_0_18px_oklch(0.78_0.15_55_/_0.25)]"
          />
        </div>
        <div className="flex overflow-hidden rounded-lg border border-border text-xs">
          {(["all", "INFO", "WARN", "ERROR"] as const).map((l) => (
            <button
              key={l}
              onClick={() => setLevel(l)}
              className={cn(
                "relative px-2.5 py-1.5 transition-colors",
                level === l ? "text-primary-foreground" : "text-muted-foreground hover:text-foreground",
              )}
            >
              {level === l && (
                <motion.span
                  layoutId="console-level-pill"
                  className="absolute inset-0 bg-primary"
                  transition={{ type: "spring", stiffness: 500, damping: 38 }}
                />
              )}
              <span className="relative">{l === "all" ? "All" : l}</span>
            </button>
          ))}
        </div>
        <Button variant={stamps ? "secondary" : "outline"} size="sm" onClick={() => setStamps((v) => !v)}>
          Timestamps
        </Button>
        <Button variant={wrap ? "secondary" : "outline"} size="sm" onClick={() => setWrap((v) => !v)}>
          Wrap
        </Button>
        <Button variant={live ? "secondary" : "outline"} size="sm" onClick={() => setLive((v) => !v)}>
          {live ? <Pause className="size-3.5" /> : <Play className="size-3.5" />}
          {live ? "Live" : "Paused"}
        </Button>
        <div className="ml-auto flex items-center gap-1.5">
          <Button variant="ghost" size="sm" onClick={copyAll}>
            <Copy className="size-3.5" /> Copy
          </Button>
          <Button variant="ghost" size="sm" onClick={download}>
            <Download className="size-3.5" /> Log
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="text-muted-foreground hover:text-destructive"
            onClick={() => {
              clear(id);
              toast.success("Console cleared");
            }}
          >
            <BrushCleaning className="size-3.5" /> Clear
          </Button>
        </div>
      </div>

      {/* Terminal */}
      <div className="sf-console-shell relative min-h-0 flex-1 overflow-hidden rounded-xl border border-zinc-800 bg-zinc-950 shadow-[0_20px_60px_-20px_oklch(0_0_0_/_0.8),0_0_40px_-18px_oklch(0.78_0.15_55_/_0.25)]">
        {/* chrome bar */}
        <div className="flex items-center gap-2 border-b border-white/[0.06] bg-white/[0.02] px-3 py-2">
          <span className="flex gap-1.5">
            <i className="size-2.5 rounded-full bg-[#ff5f57]" />
            <i className="size-2.5 rounded-full bg-[#febc2e]" />
            <i className="size-2.5 rounded-full bg-[#28c840]" />
          </span>
          <span className="ml-1 flex items-center gap-1.5 font-mono text-[11px] text-zinc-400">
            <TerminalSquare className="size-3.5 text-zinc-500" />
            {server?.name ?? "console"} — live terminal
          </span>
          <span
            className={cn(
              "ml-auto flex items-center gap-1.5 rounded-full px-2 py-0.5 font-mono text-[10px]",
              online ? "bg-emerald-500/10 text-emerald-300" : "bg-zinc-500/10 text-zinc-400",
            )}
          >
            <i className={cn("size-1.5 rounded-full", online ? "sf-blink bg-emerald-400" : "bg-zinc-500")} />
            {online ? `● ${lines.length} lines` : `○ ${lines.length} lines`}
          </span>
        </div>

        <div
          ref={scroller}
          onScroll={onScroll}
          className={cn(
            "sf-console-scroll min-h-0 h-[calc(100%-37px)] overflow-auto p-3 font-mono text-[12px] leading-5 text-zinc-100",
            wrap ? "whitespace-pre-wrap" : "whitespace-pre",
          )}
        >
          {visible.rows.length === 0 ? (
            <div className="flex h-full min-h-[240px] flex-col items-center justify-center gap-2 text-center">
              <motion.div
                initial={reduce ? false : { scale: 0.9, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                transition={{ type: "spring", stiffness: 260, damping: 22 }}
                className="grid size-14 place-items-center rounded-2xl border border-white/10 bg-white/[0.03]"
              >
                <TerminalSquare className="size-7 text-zinc-500" />
              </motion.div>
              <p className="text-sm text-zinc-300">No console output yet.</p>
              <p className="max-w-sm text-xs text-zinc-500">
                {online
                  ? "The server is running. Type a command below — it will echo here with the response."
                  : "Start the server from Overview to stream live logs into this terminal."}
              </p>
            </div>
          ) : (
            <>
              {visible.capped > 0 && (
                <div className="mb-2 rounded-lg border border-white/10 bg-white/[0.03] px-2 py-1 text-center text-[11px] text-zinc-500">
                  Showing latest {RENDER_CAP} of {filtered.length} matching lines — refine search to see more.
                </div>
              )}
              {visible.rows.map((line, i) => (
                <LogRow
                  key={`${line.timestamp}-${i}-${line.line.slice(0, 24)}`}
                  line={line}
                  showStamp={stamps}
                  animate={i >= visible.rows.length - 40}
                />
              ))}
              {/* live caret */}
              {live && online && (
                <div className="flex items-center gap-1 px-2 py-1 text-zinc-500">
                  <ChevronRight className="size-3.5 text-emerald-500/80" />
                  <span className="sf-caret inline-block h-3.5 w-[7px] bg-emerald-400/80" />
                </div>
              )}
            </>
          )}
        </div>

        {/* scanline sheen */}
        <div className="sf-scanlines pointer-events-none absolute inset-0" />

        {/* jump to latest */}
        <AnimatePresence>
          {!pinned && visible.rows.length > 0 && (
            <motion.button
              initial={{ opacity: 0, y: 8, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 8, scale: 0.95 }}
              onClick={jumpLatest}
              className="absolute bottom-4 left-1/2 flex -translate-x-1/2 items-center gap-1.5 rounded-full border border-primary/40 bg-background/90 px-3 py-1.5 text-xs shadow-lg backdrop-blur transition-transform hover:scale-105"
            >
              <ArrowDownToLine className="size-3.5" /> Jump to latest
            </motion.button>
          )}
        </AnimatePresence>
      </div>

      {/* Command bar */}
      <form
        className="flex shrink-0 gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          void send(command);
        }}
      >
        <div className="sf-command-glow group relative flex-1">
          <span className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 font-mono text-sm font-bold text-emerald-400">
            ›
          </span>
          <Input
            ref={inputRef}
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder={online ? "Type a command — say Hello everyone!  (↑ history)" : "Start the server to send commands…"}
            disabled={!online || sending}
            className="bg-zinc-950 pl-7 font-mono transition-shadow placeholder:text-zinc-600 focus-visible:shadow-[0_0_0_1px_var(--ring),0_0_24px_oklch(0.72_0.18_150_/_0.25)]"
            autoComplete="off"
            spellCheck={false}
          />
        </div>
        <motion.div whileTap={reduce ? undefined : { scale: 0.96 }}>
          <Button type="submit" disabled={!online || sending || !command.trim()} className="min-w-20">
            {sending ? "Sending…" : "Send"}
          </Button>
        </motion.div>
      </form>
    </motion.div>
  );
}
