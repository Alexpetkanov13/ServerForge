import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export type UpdateMeta = {
  version: string;
  currentVersion: string;
  date?: string;
  body?: string;
};

export function metaOf(update: Update): UpdateMeta {
  return {
    version: update.version,
    currentVersion: update.currentVersion,
    date: update.date,
    body: update.body ?? undefined,
  };
}

/**
 * Check for updates. Never throws — returns null when there is nothing to
 * do (up to date, offline, unconfigured feed, or plain browser preview).
 * Pass manual=true to get the underlying error back for user feedback.
 */
export async function checkForUpdates(manual = false): Promise<{ update: Update | null; error?: string }> {
  try {
    const update = await check({ timeout: 20_000 });
    return { update };
  } catch (err) {
    const raw = err instanceof Error ? err.message : String(err);
    if (manual) return { update: null, error: friendlyUpdateError(raw) };
    return { update: null };
  }
}

/** Translate plugin errors into something a human can act on. */
export function friendlyUpdateError(raw: string): string {
  if (/valid release json|404|not found/i.test(raw)) {
    return "No published releases found yet — publish a release on GitHub to feed the updater.";
  }
  if (/network|dns|failed to fetch|timed out|timeout|offline|connection/i.test(raw)) {
    return "Couldn't reach the update server. Check your internet connection and try again.";
  }
  if (/signature|public key/i.test(raw)) {
    return "The update couldn't be verified (signature mismatch). It was blocked for your safety.";
  }
  return raw;
}

/** Fold plugin download events into (downloadedBytes, totalBytes|null). */
export function downloadProgress(
  onProgress: (downloaded: number, total: number | null) => void,
): (event: DownloadEvent) => void {
  let downloaded = 0;
  let total: number | null = null;
  return (event) => {
    if (event.event === "Started") {
      downloaded = 0;
      total = event.data.contentLength ?? null;
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
    }
    onProgress(downloaded, total);
  };
}

export async function downloadOnly(
  update: Update,
  onProgress: (downloaded: number, total: number | null) => void,
): Promise<void> {
  await update.download(downloadProgress(onProgress));
}

/** Install an already-downloaded package. On Windows this launches the installer and exits the app. */
export async function installDownloaded(update: Update): Promise<void> {
  await update.install({ restartAfterInstall: true });
}
