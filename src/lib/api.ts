import { invoke } from "@tauri-apps/api/core";
import type {
  AddonInfo,
  AddonVersion,
  AppSettings,
  BackupRecord,
  BuildInfo,
  CompatibilityInfo,
  CreateServerRequest,
  DiscoveredServer,
  DiskInfo,
  DownloadProgress,
  FileEntry,
  InstalledAddon,
  JavaRuntime,
  MinecraftVersionInfo,
  ProviderInfo,
  ServerProperties,
  ServerRecord,
  SharePackInfo,
  SystemInfo,
  WorldInfo,
} from "@/types";

export const api = {
  providerCatalog: () => invoke<ProviderInfo[]>("provider_catalog"),
  versions: (provider: string) =>
    invoke<MinecraftVersionInfo[]>("list_minecraft_versions", { provider }),
  builds: (provider: string, minecraftVersion: string) =>
    invoke<BuildInfo[]>("list_builds", { provider, minecraftVersion }),
  compatibility: (provider: string, minecraftVersion: string) =>
    invoke<CompatibilityInfo>("provider_compatibility", { provider, minecraftVersion }),
  servers: () => invoke<ServerRecord[]>("list_servers"),
  server: (id: string) => invoke<ServerRecord>("get_server", { id }),
  validateName: (name: string) => invoke<void>("validate_name", { name }),
  inspectDirectory: (path: string) => invoke<DiskInfo>("inspect_directory", { path }),
  createServer: (request: CreateServerRequest) => invoke<string>("create_server", { request }),
  cleanupInstallation: (serverId: string, deleteFiles: boolean) =>
    invoke<void>("cleanup_installation", { serverId, deleteFiles }),
  start: (id: string) => invoke<void>("start_server", { id }),
  stop: (id: string, force = false) => invoke<void>("stop_server", { id, force }),
  restart: (id: string) => invoke<void>("restart_server", { id }),
  command: (id: string, command: string) => invoke<void>("send_command", { id, command }),
  deleteServer: (id: string, deleteFiles: boolean) =>
    invoke<void>("delete_server", { id, deleteFiles }),
  detectJava: () => invoke<JavaRuntime[]>("detect_java"),
  testJava: (path: string) => invoke("test_java", { path }),
  javaDownloadUrl: (major: number) => invoke<string>("java_download_url", { major }),
  system: () => invoke<SystemInfo>("system_snapshot"),
  recommendedJava: (minecraftVersion: string) =>
    invoke<number>("recommended_java", { minecraftVersion }),
  memoryAdvice: (allocatedMb: number, totalSystemMb: number) =>
    invoke<string | null>("memory_advice", { allocatedMb, totalSystemMb }),
  portInUse: (port: number) => invoke<boolean>("port_in_use", { port }),
  files: (id: string, rel: string) => invoke<FileEntry[]>("list_files", { id, rel }),
  readFile: (id: string, rel: string) => invoke<string>("read_file", { id, rel }),
  writeFile: (id: string, rel: string, contents: string) =>
    invoke<void>("write_file", { id, rel, contents }),
  createFolder: (id: string, rel: string) => invoke<void>("create_folder", { id, rel }),
  renameFile: (id: string, from: string, to: string) =>
    invoke<void>("rename_file", { id, from, to }),
  deleteFile: (id: string, rel: string) => invoke<void>("delete_file", { id, rel }),
  searchFiles: (id: string, query: string) => invoke<FileEntry[]>("search_files", { id, query }),
  uploadInto: (id: string, rel: string, sources: string[]) =>
    invoke<number>("upload_into", { id, rel, sources }),
  properties: (id: string) => invoke<ServerProperties>("get_properties", { id }),
  saveProperties: (id: string, updates: Record<string, string>) =>
    invoke<void>("save_properties", { id, updates }),
  backups: (id: string) => invoke<BackupRecord[]>("list_backups", { id }),
  createBackup: (id: string, trigger = "manual") =>
    invoke<BackupRecord>("create_backup", { id, trigger }),
  restoreBackup: (id: string, backupId: string) =>
    invoke<void>("restore_backup", { id, backupId }),
  deleteBackup: (backupId: string) => invoke<void>("delete_backup", { backupId }),
  worlds: (id: string) => invoke<WorldInfo[]>("list_worlds", { id }),
  addons: (id: string, kind: "plugins" | "mods") =>
    invoke<InstalledAddon[]>("list_addons", { id, kind }),
  searchPlugins: (query: string, minecraftVersion: string, provider: string) =>
    invoke<AddonInfo[]>("search_plugins", { query, minecraftVersion, provider }),
  searchMods: (query: string, loader: string, minecraftVersion?: string) =>
    invoke<AddonInfo[]>("search_mods", { query, loader, minecraftVersion: minecraftVersion ?? null }),
  pluginVersions: (author: string, slug: string, minecraftVersion?: string, platform?: string) =>
    invoke<AddonVersion[]>("plugin_versions", { author, slug, minecraftVersion: minecraftVersion ?? null, platform: platform ?? null }),
  resolvePluginDownload: (payload: {
    source: string;
    id: string;
    author: string;
    slug: string;
    minecraftVersion: string;
    provider: string;
  }) => invoke<AddonVersion>("resolve_plugin_download", payload),
  modVersions: (id: string, loader: string, game: string) =>
    invoke<AddonVersion[]>("mod_versions", { id, loader, game }),
  installAddon: (payload: {
    serverId: string;
    kind: string;
    url: string;
    fileName: string;
    label: string;
  }) => invoke<void>("install_addon", payload),
  toggleAddon: (path: string, enabled: boolean) => invoke<void>("toggle_addon", { path, enabled }),
  removeAddon: (path: string) => invoke<void>("remove_addon", { path }),
  scan: (path: string) => invoke<DiscoveredServer[]>("scan_servers", { path }),
  importServer: (discovered: DiscoveredServer) =>
    invoke<ServerRecord>("import_server", { discovered }),
  importSharePack: (packPath: string, destDir?: string | null) =>
    invoke<ServerRecord>("import_share_pack", { packPath, destDir: destDir ?? null }),
  sharePack: (id: string) => invoke<SharePackInfo>("get_share_pack", { id }),
  settings: () => invoke<AppSettings>("load_settings"),
  saveSettings: (value: AppSettings) => invoke<void>("save_settings", { value }),
  setAutoStart: (id: string, autoStart: boolean) =>
    invoke<void>("set_auto_start", { id, autoStart }),
  updateMemory: (id: string, minMb: number, maxMb: number) =>
    invoke<void>("update_memory", { id, minMb, maxMb }),
  downloads: () => invoke<DownloadProgress[]>("list_downloads"),
  openPath: (path: string) => invoke<void>("open_path", { path }),
  openLogs: () => invoke<string>("open_logs"),
  diagnostic: () => invoke<string>("diagnostic_report"),
  defaultPath: (name: string) => invoke<string>("default_server_path", { name }),
};

export function errorMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err && typeof err === "object") {
    const e = err as { message?: string; title?: string };
    return e.message || e.title || "Something went wrong.";
  }
  return "Something went wrong.";
}

export function errorDetails(err: unknown) {
  if (err && typeof err === "object") {
    return err as { title?: string; message?: string; causes?: string[]; technical?: string };
  }
  return { message: errorMessage(err), causes: [], technical: String(err) };
}
