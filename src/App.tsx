import { useEffect } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { MotionConfig } from "framer-motion";
import { ThemeProvider } from "next-themes";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";
import { AppShell } from "@/components/layout/AppShell";
import { ForgeLoader } from "@/components/layout/ForgeLoader";
import { DashboardPage } from "@/pages/Dashboard";
import { ServersPage } from "@/pages/Servers";
import { CreateServerPage } from "@/pages/CreateServer";
import { TemplatesPage } from "@/pages/Templates";
import { DownloadsPage } from "@/pages/Downloads";
import { JavaPage } from "@/pages/Java";
import { SettingsPage } from "@/pages/Settings";
import { OnboardingPage } from "@/pages/Onboarding";
import { ServerLayout } from "@/pages/server/ServerLayout";
import { OverviewPage } from "@/pages/server/Overview";
import { ConsolePage } from "@/pages/server/Console";
import { AddonsPage } from "@/pages/server/Addons";
import { FilesPage } from "@/pages/server/Files";
import { WorldsPage } from "@/pages/server/Worlds";
import { BackupsPage } from "@/pages/server/Backups";
import { ServerSettingsPage } from "@/pages/server/ServerSettings";
import { PlayersPage, UpdatesPage } from "@/pages/server/Updates";
import { useAppStore } from "@/stores/useAppStore";

function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/servers" element={<ServersPage />} />
        <Route path="/servers/:id" element={<ServerLayout />}>
          <Route index element={<OverviewPage />} />
          <Route path="console" element={<ConsolePage />} />
          <Route path="players" element={<PlayersPage />} />
          <Route path="plugins" element={<AddonsPage kind="plugins" />} />
          <Route path="mods" element={<AddonsPage kind="mods" />} />
          <Route path="files" element={<FilesPage />} />
          <Route path="worlds" element={<WorldsPage />} />
          <Route path="backups" element={<BackupsPage />} />
          <Route path="settings" element={<ServerSettingsPage />} />
          <Route path="updates" element={<UpdatesPage />} />
        </Route>
        <Route path="/create" element={<CreateServerPage />} />
        <Route path="/templates" element={<TemplatesPage />} />
        <Route path="/downloads" element={<DownloadsPage />} />
        <Route path="/java" element={<JavaPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}

export default function App() {
  const settings = useAppStore((s) => s.settings);
  const ready = useAppStore((s) => s.ready);
  const load = useAppStore((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false}>
      <MotionConfig reducedMotion={settings?.reducedMotion ? "always" : "user"}>
        <TooltipProvider>
          {!ready ? (
            <ForgeLoader />
          ) : (
            <BrowserRouter>
              {settings && !settings.onboarded ? <OnboardingPage /> : <AppRoutes />}
            </BrowserRouter>
          )}
          <Toaster />
        </TooltipProvider>
      </MotionConfig>
    </ThemeProvider>
  );
}
