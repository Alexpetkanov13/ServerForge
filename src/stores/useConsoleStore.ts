import { create } from "zustand";
import type { ServerLogLine } from "@/types";

const MAX_LINES = 2000;

type ConsoleState = {
  lines: Record<string, ServerLogLine[]>;
  /** Tracks servers already seeded from latest.log so a cleared view never reappears on tab switch. */
  seeded: Record<string, boolean>;
  push: (line: ServerLogLine) => void;
  /** Echo a typed command (and optional error) so it is always visible in the console. */
  echo: (serverId: string, command: string) => void;
  pushError: (serverId: string, message: string) => void;
  seed: (serverId: string, text: string) => void;
  clear: (serverId: string) => void;
};

function makeLine(serverId: string, stream: string, line: string): ServerLogLine {
  return { serverId, stream, line, timestamp: new Date().toISOString() };
}

function append(lines: ServerLogLine[], line: ServerLogLine): ServerLogLine[] {
  if (lines.length >= MAX_LINES) {
    // Avoid spreading 2000 items on every log line — mutate a copied window instead.
    const next = lines.slice(lines.length - MAX_LINES + 1);
    next.push(line);
    return next;
  }
  const next = lines.slice();
  next.push(line);
  return next;
}

export const useConsoleStore = create<ConsoleState>((set) => ({
  lines: {},
  seeded: {},
  push: (line) =>
    set((s) => {
      const prev = s.lines[line.serverId] ?? [];
      return { lines: { ...s.lines, [line.serverId]: append(prev, line) } };
    }),
  echo: (serverId, command) =>
    set((s) => {
      const prev = s.lines[serverId] ?? [];
      return {
        seeded: s.seeded[serverId] ? s.seeded : { ...s.seeded, [serverId]: true },
        lines: { ...s.lines, [serverId]: append(prev, makeLine(serverId, "stdin", `> ${command}`)) },
      };
    }),
  pushError: (serverId, message) =>
    set((s) => {
      const prev = s.lines[serverId] ?? [];
      return {
        lines: { ...s.lines, [serverId]: append(prev, makeLine(serverId, "stderr", message)) },
      };
    }),
  seed: (serverId, text) =>
    set((s) => {
      // Seed exactly once per server session. This is what keeps "Clear" persistent:
      // remounting the console tab must not re-hydrate old file content.
      if (s.seeded[serverId]) return s;
      const seeded = text
        .split(/\r?\n/)
        .filter((line) => line.length > 0)
        .slice(-MAX_LINES)
        .map((line) => ({
          serverId,
          stream: "stdout",
          line,
          timestamp: "",
        }));
      return {
        seeded: { ...s.seeded, [serverId]: true },
        lines: { ...s.lines, [serverId]: seeded },
      };
    }),
  clear: (serverId) =>
    set((s) => ({
      // Keep seeded=true so switching tabs does NOT resurrect cleared content.
      seeded: { ...s.seeded, [serverId]: true },
      lines: { ...s.lines, [serverId]: [] },
    })),
}));
