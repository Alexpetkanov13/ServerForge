export const easeOut = [0.22, 1, 0.36, 1] as const;

export const pageTransition = {
  duration: 0.32,
  ease: easeOut,
};

export const springSnappy = {
  type: "spring",
  stiffness: 480,
  damping: 36,
} as const;

export const springSoft = {
  type: "spring",
  stiffness: 260,
  damping: 26,
} as const;

/** Direction-aware wizard step transition. Pass custom={direction} (+1 forward, -1 back). */
export const wizardVariants = {
  enter: (dir: number) => ({ opacity: 0, x: 48 * dir, scale: 0.985 }),
  center: { opacity: 1, x: 0, scale: 1 },
  exit: (dir: number) => ({ opacity: 0, x: -48 * dir, scale: 0.985 }),
};

export const staggerParent = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.06, delayChildren: 0.05 },
  },
};

export const staggerChild = {
  hidden: { opacity: 0, y: 18, scale: 0.98 },
  show: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { duration: 0.34, ease: easeOut },
  },
};

export const listVariants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.055, delayChildren: 0.04 },
  },
};

export const itemVariants = {
  hidden: { opacity: 0, y: 12 },
  show: {
    opacity: 1,
    y: 0,
    transition: { duration: 0.32, ease: easeOut },
  },
};

export function shellKey(pathname: string) {
  const parts = pathname.split("/").filter(Boolean);
  if (parts[0] === "servers" && parts[1]) return `/servers/${parts[1]}`;
  return pathname;
}
