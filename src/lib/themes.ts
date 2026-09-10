export type ThemeId = "ember" | "midnight" | "forest" | "light";

export const THEME_IDS: ThemeId[] = ["ember", "midnight", "forest", "light"];

/** Legacy installs stored "dark" — that is ember. Anything unknown is ember. */
export function normalizeTheme(raw?: string | null): ThemeId {
  if (raw === "midnight" || raw === "forest" || raw === "light") return raw;
  return "ember";
}

export const THEMES: Array<{ id: ThemeId; label: string; desc: string; swatch: string }> = [
  {
    id: "ember",
    label: "Ember",
    desc: "Signature dark",
    swatch: "linear-gradient(135deg, #f59e0b 0%, #78350a 60%, #1c0a00 100%)",
  },
  {
    id: "midnight",
    label: "Midnight",
    desc: "Cool blue dark",
    swatch: "linear-gradient(135deg, #60a5fa 0%, #1e3a8a 60%, #020617 100%)",
  },
  {
    id: "forest",
    label: "Forest",
    desc: "Fresh green dark",
    swatch: "linear-gradient(135deg, #34d399 0%, #065f46 60%, #02120c 100%)",
  },
  {
    id: "light",
    label: "Daylight",
    desc: "Bright light",
    swatch: "linear-gradient(135deg, #ffffff 0%, #e7e5e4 60%, #a8a29e 100%)",
  },
];

/**
 * Full-surface variable overrides applied as *inline styles* on <html>.
 * Inline custom properties beat every stylesheet rule, so themed surfaces
 * apply deterministically in any WebView. Ember and light use the
 * stylesheet defaults (empty map = nothing to override).
 */
export const THEME_VARS: Record<ThemeId, Record<string, string>> = {
  ember: {},
  light: {},
  midnight: {
    "--background": "oklch(0.17 0.025 260)",
    "--foreground": "oklch(0.94 0.015 250)",
    "--card": "oklch(0.21 0.03 260)",
    "--card-foreground": "oklch(0.94 0.015 250)",
    "--popover": "oklch(0.21 0.03 260)",
    "--popover-foreground": "oklch(0.94 0.015 250)",
    "--primary": "oklch(0.72 0.14 250)",
    "--primary-foreground": "oklch(0.16 0.03 260)",
    "--secondary": "oklch(0.26 0.03 260)",
    "--secondary-foreground": "oklch(0.94 0.015 250)",
    "--muted": "oklch(0.25 0.025 260)",
    "--muted-foreground": "oklch(0.72 0.04 260)",
    "--accent": "oklch(0.27 0.04 260)",
    "--accent-foreground": "oklch(0.94 0.015 250)",
    "--border": "oklch(0.9 0.03 250 / 12%)",
    "--input": "oklch(0.9 0.03 250 / 14%)",
    "--ring": "oklch(0.72 0.14 250)",
    "--chart-1": "oklch(0.72 0.14 250)",
    "--chart-2": "oklch(0.65 0.1 255)",
    "--sidebar": "oklch(0.15 0.025 260)",
    "--sidebar-foreground": "oklch(0.94 0.015 250)",
    "--sidebar-primary": "oklch(0.72 0.14 250)",
    "--sidebar-primary-foreground": "oklch(0.16 0.03 260)",
    "--sidebar-accent": "oklch(0.25 0.035 260)",
    "--sidebar-accent-foreground": "oklch(0.94 0.015 250)",
    "--sidebar-border": "oklch(0.9 0.03 250 / 10%)",
    "--sidebar-ring": "oklch(0.72 0.14 250)",
    "--sf-glow-1": "oklch(0.5 0.14 250 / 0.3)",
    "--sf-glow-2": "oklch(0.4 0.1 255 / 0.22)",
    "--sf-glow-3": "oklch(0.35 0.08 250 / 0.14)",
  },
  forest: {
    "--background": "oklch(0.16 0.02 155)",
    "--foreground": "oklch(0.94 0.015 155)",
    "--card": "oklch(0.2 0.03 155)",
    "--card-foreground": "oklch(0.94 0.015 155)",
    "--popover": "oklch(0.2 0.03 155)",
    "--popover-foreground": "oklch(0.94 0.015 155)",
    "--primary": "oklch(0.78 0.17 155)",
    "--primary-foreground": "oklch(0.15 0.04 155)",
    "--secondary": "oklch(0.25 0.03 155)",
    "--secondary-foreground": "oklch(0.94 0.015 155)",
    "--muted": "oklch(0.24 0.025 155)",
    "--muted-foreground": "oklch(0.74 0.04 155)",
    "--accent": "oklch(0.26 0.04 155)",
    "--accent-foreground": "oklch(0.94 0.015 155)",
    "--border": "oklch(0.9 0.03 155 / 12%)",
    "--input": "oklch(0.9 0.03 155 / 14%)",
    "--ring": "oklch(0.78 0.17 155)",
    "--chart-1": "oklch(0.78 0.17 155)",
    "--chart-2": "oklch(0.7 0.12 160)",
    "--sidebar": "oklch(0.14 0.02 155)",
    "--sidebar-foreground": "oklch(0.94 0.015 155)",
    "--sidebar-primary": "oklch(0.78 0.17 155)",
    "--sidebar-primary-foreground": "oklch(0.15 0.04 155)",
    "--sidebar-accent": "oklch(0.24 0.035 155)",
    "--sidebar-accent-foreground": "oklch(0.94 0.015 155)",
    "--sidebar-border": "oklch(0.9 0.03 155 / 10%)",
    "--sidebar-ring": "oklch(0.78 0.17 155)",
    "--sf-glow-1": "oklch(0.55 0.15 155 / 0.3)",
    "--sf-glow-2": "oklch(0.45 0.11 160 / 0.22)",
    "--sf-glow-3": "oklch(0.38 0.08 155 / 0.14)",
  },
};

const MANAGED_VARS = Array.from(
  new Set(Object.values(THEME_VARS).flatMap((vars) => Object.keys(vars))),
);

/** Apply a theme: stylesheet class + data attr + bulletproof inline variables. */
export function applyTheme(raw?: string | null) {
  const theme = normalizeTheme(raw);
  const root = document.documentElement;
  root.classList.toggle("dark", theme !== "light");
  root.dataset.theme = theme;
  for (const key of MANAGED_VARS) root.style.removeProperty(key);
  const vars = THEME_VARS[theme];
  for (const [key, value] of Object.entries(vars)) root.style.setProperty(key, value);
  root.style.colorScheme = theme === "light" ? "light" : "dark";
  return theme;
}
