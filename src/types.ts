export type AppError = {
  title: string;
  message: string;
  causes: string[];
  technical: string;
};

export type ProviderInfo = {
  id: string;
  name: string;
  description: string;
  recommendedUse: string;
  ecosystem: "vanilla" | "bukkit" | "modded";
  supportsPlugins: boolean;
  supportsMods: boolean;
  performance: string;
};

export type MinecraftVersionInfo = {
  id: string;
  channel: string;
  recommended: boolean;
  latest: boolean;
  javaMajor: number;
  released?: string | null;
};

export type BuildInfo = {
  id: string;
  label: string;
  channel: string;
  recommended: boolean;
  released?: string | null;
  loaderVersion?: string | null;
  installerVersion?: string | null;
};

export type CompatibilityInfo = {
  provider: string;
  minecraftVersion: string;
  supported: boolean;
  stableBuild: boolean;
  javaMajor: number;
  notes: string[];
};

export type ServerRecord = {
  id: string;
  name: string;
  path: string;
  provider: string;
  minecraftVersion: string;
  loaderVersion?: string | null;
  build?: string | null;
  javaPath?: string | null;
  memoryMinMb: number;
  memoryMaxMb: number;
  port: number;
  status: ServerStatus;
  autoStart: number;
  eulaAccepted: number;
  motd?: string | null;
  pid?: number | null;
  createdAt: string;
  updatedAt: string;
  lastStartedAt?: string | null;
};

export type SharePackInfo = {
  path: string;
  sizeBytes: number;
  updatedAt: string;
};

export type ServerStatus =
  | "offline"
  | "starting"
  | "online"
  | "stopping"
  | "crashed"
  | "installing"
  | "updating"
  | "error";

export type CreateServerRequest = {
  name: string;
  path: string;
  provider: string;
  minecraftVersion: string;
  build?: string | null;
  loaderVersion?: string | null;
  installerVersion?: string | null;
  javaPath?: string | null;
  memoryMinMb: number;
  memoryMaxMb: number;
  port: number;
  motd?: string | null;
  eulaAccepted: boolean;
  startAfterInstall: boolean;
  generateWorld: boolean;
  optimizedFlags: boolean;
  difficulty?: string | null;
  gamemode?: string | null;
  maxPlayers?: number | null;
  onlineMode?: boolean | null;
  pvp?: boolean | null;
  viewDistance?: number | null;
  simulationDistance?: number | null;
  allowFlight?: boolean | null;
};

export type JavaRuntime = {
  id: string;
  path: string;
  version: string;
  major: number;
  vendor: string;
  architecture: string;
  isManaged: boolean;
  compatible: boolean;
};

export type SystemInfo = {
  totalMemoryMb: number;
  availableMemoryMb: number;
  cpuCount: number;
  os: string;
  arch: string;
  appVersion: string;
  offline: boolean;
};

export type DiskInfo = {
  path: string;
  totalBytes: number;
  availableBytes: number;
  writable: boolean;
  containsServer: boolean;
  empty: boolean;
};

export type ServerMetrics = {
  serverId: string;
  cpuPercent: number;
  memoryBytes: number;
  memoryMaxBytes: number;
  uptimeSeconds: number;
  playerCount?: number | null;
  maxPlayers?: number | null;
  tps?: number | null;
  status: string;
};

export type ServerLogLine = {
  serverId: string;
  stream: string;
  line: string;
  timestamp: string;
};

export type InstallationProgress = {
  id: string;
  serverId?: string | null;
  step: string;
  stepIndex: number;
  stepCount: number;
  status: string;
  message: string;
  percent: number;
  logs: string[];
  error?: AppError | null;
};

export type DownloadProgress = {
  id: string;
  label: string;
  status: string;
  bytesDownloaded: number;
  bytesTotal?: number | null;
  speedBps: number;
  etaSeconds?: number | null;
  error?: string | null;
};

export type FileEntry = {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  modified?: string | null;
};

export type PropertyField = {
  key: string;
  label: string;
  value: string;
  kind: string;
  options: string[];
  hint?: string | null;
};

export type ServerProperties = {
  fields: PropertyField[];
  raw: string;
};

export type BackupRecord = {
  id: string;
  serverId: string;
  label?: string | null;
  path: string;
  sizeBytes: number;
  trigger: string;
  createdAt: string;
};

export type WorldInfo = {
  name: string;
  path: string;
  sizeBytes: number;
  kind: string;
};

export type AddonInfo = {
  id: string;
  slug: string;
  name: string;
  description: string;
  author: string;
  downloads: number;
  iconUrl?: string | null;
  platform: string;
  source: string;
  gameVersions?: string[];
  compatible?: boolean;
};

export type AddonVersion = {
  id: string;
  name: string;
  versionNumber: string;
  gameVersions: string[];
  loaders: string[];
  downloadUrl?: string | null;
  fileName?: string | null;
  sha512?: string | null;
  requiredDependencies: string[];
};

export type InstalledAddon = {
  name: string;
  fileName: string;
  enabled: boolean;
  version?: string | null;
  path: string;
};

export type DiscoveredServer = {
  path: string;
  name: string;
  provider?: string | null;
  minecraftVersion?: string | null;
  pluginCount: number;
  modCount: number;
  worldCount: number;
  hasEula: boolean;
  port?: number | null;
};

export type AppSettings = {
  defaultServersDir: string;
  preferredJavaPath?: string | null;
  theme: string;
  reducedMotion: boolean;
  fontScale: number;
  developerMode: boolean;
  backupRetention: number;
  autoUpdates: boolean;
  onboarded: boolean;
  appAutoUpdate: boolean;
  appAutoInstall: boolean;
  skippedAppVersion?: string | null;
};

export type CrashInfo = {
  serverId: string;
  reason: string;
  suggestion: string;
  reportPath?: string | null;
};
