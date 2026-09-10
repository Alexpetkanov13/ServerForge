import { useEffect } from "react";
import { UpdateDialog } from "@/components/layout/UpdateDialog";
import { useAppStore } from "@/stores/useAppStore";
import { useUpdateStore } from "@/stores/useUpdateStore";

const LAUNCH_CHECK_DELAY_MS = 4000;

/** Mount once (in AppShell): silent launch check + the update popup. */
export function UpdateManager() {
  const autoUpdate = useAppStore((s) => s.settings?.appAutoUpdate);
  const checked = useUpdateStore((s) => s.checkedThisSession);
  const checkNow = useUpdateStore((s) => s.checkNow);

  useEffect(() => {
    // Wait for the app to settle so launch stays snappy; never blocks UI.
    if (!autoUpdate || checked) return;
    const timer = setTimeout(() => {
      void checkNow(false);
    }, LAUNCH_CHECK_DELAY_MS);
    return () => clearTimeout(timer);
  }, [autoUpdate, checked, checkNow]);

  return <UpdateDialog />;
}
