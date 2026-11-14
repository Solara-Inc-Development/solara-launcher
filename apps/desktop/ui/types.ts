export type PageId =
  | 'home'
  | 'library'
  | 'discover'
  | 'skins'
  | 'screenshots'
  | 'solara'
  | 'tasks'
  | 'settings'
  | 'instance'
  | 'account';

export type InstanceDetailTab =
  | 'overview'
  | 'content'
  | 'files'
  | 'worlds'
  | 'logs'
  | 'screenshots'
  | 'settings';

export type LoaderKind = 'vanilla' | 'fabric' | 'quilt' | 'forge' | 'neoforge';

export interface InstanceRecord {
  schemaVersion: number;
  id: string;
  name: string;
  gameVersion: string;
  loader: LoaderKind;
  loaderVersion: string | null;
  memory: number;
  javaPath: string;
  state: 'created' | 'installing' | 'installed' | 'failed' | 'cancelled';
  running?: boolean;
  icon?: string | null;
  customIcon?: string | null;
  group?: string | null;
  pinned?: boolean;
  favourite?: boolean;
  createdAt?: string | null;
  lastPlayed?: string | null;
  playtimeSeconds?: number;
  jvmArgs?: string | null;
  resolution?: [number, number] | null;
  fullscreen?: boolean | null;
  preLaunch?: string | null;
  postExit?: string | null;
  updateChannel?: 'release' | 'beta' | 'alpha' | null;
  notes?: string | null;
}

export interface MinecraftAccount {
  id: string;
  username: string;
  selected: boolean;
}

export interface DownloadJob {
  id: string;
  instance: string;
  name: string;
  kind: string;
  status: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';
  phase: string;
  done: number;
  total: number;
  createdAt?: string;
  actions: string[];
}

export interface LauncherPreferences {
  prefsVersion?: number;
  memory: number;
  javaPath: string;
  theme: 'obsidian-blue' | 'dawn-gold' | 'slate' | 'midnight';
  uiScale: number;
  reducedMotion: boolean;
  density: 'comfortable' | 'compact';
  onLaunch: 'keep' | 'minimize';
  confirmLaunch: boolean;
  maxConcurrentDownloads: number;
  defaultWidth: number;
  defaultHeight: number;
  fullscreen: boolean;
  jvmArgs: string;
  diagnosticsEnabled: boolean;
  announcementsEnabled: boolean;
  solaraHubEnabled: boolean;
  autoUpdateEnabled: boolean;
  onboardingCompleted: boolean;
  groups: string[];
}

export interface ModEntry {
  path: string;
  fileName: string;
  name: string;
  version: string | null;
  modId: string | null;
  loaders?: string[];
  issue?: string | null;
  enabled: boolean;
  size: number;
}

export interface PackEntry {
  kind: string;
  path: string;
  fileName: string;
  name: string;
  description: string | null;
  packFormat: number | null;
  iconDataUrl: string | null;
  enabled: boolean;
  directory: boolean;
  size: number;
  world: string | null;
}

export interface ContentUpdateItem {
  currentPath: string;
  currentFileName: string;
  projectId: string;
  projectName: string;
  currentVersionId: string;
  currentVersionNumber: string;
  latestVersionId: string;
  latestVersionNumber: string;
  latestVersionName: string;
  releaseChannel: string;
  changelog: string;
  destination: string;
  downloadUrl: string;
  sha1: string;
  sha512: string;
  size: number;
}

export interface FileEntry {
  name: string;
  path: string;
  directory: boolean;
  size: number;
  editable: boolean;
  image: boolean;
}

export interface TrashEntry {
  id: string;
  name: string;
  originalPath: string;
  deletedAt: string;
  size: number;
  directory: boolean;
}

export interface WorldEntry {
  folder: string;
  name: string;
  gameMode: string;
  difficulty: string;
  versionName: string;
  lastPlayedMs: number;
  size: number;
  iconDataUrl: string | null;
  locked: boolean;
  datapackCount: number;
}

export interface ScreenshotEntry {
  instanceId: string;
  instanceName: string;
  fileName: string;
  path: string;
  fullPath: string;
  size: number;
  modifiedMs: number;
}

export interface LogFileEntry {
  name: string;
  size: number;
  modifiedMs: number;
  gzipped: boolean;
}

export interface BackupEntry {
  id: string;
  name: string;
  createdAt: string;
  size: number;
  instanceId: string;
}

export interface DetectedJava {
  path: string;
  version: string;
  major: number | null;
  vendor: string;
  managed: boolean;
}

export interface StorageSummary {
  totalBytes: number;
  instancesBytes: number;
  cacheBytes: number;
  javaBytes: number;
  skinsBytes: number;
}

export interface SavedSkin {
  id: string;
  name: string;
  variant: 'classic' | 'slim';
  pngDataUrl: string;
  updatedAt: string;
}

export interface ProfileCape {
  id: string;
  alias: string;
  state: 'ACTIVE' | 'INACTIVE';
  url: string;
  dataUrl?: string | null;
}

export interface ProfileSkin {
  id: string;
  variant: 'CLASSIC' | 'SLIM';
  state: 'ACTIVE' | 'INACTIVE';
  url: string;
  dataUrl?: string | null;
}

export interface UpdateStatus {
  stage: string;
  version: string | null;
  publicVersion?: string | null;
  currentVersion?: string | null;
  notes?: string | null;
  downloaded?: number;
  total?: number | null;
  error: string | null;
  checkedAt?: string | null;
}

export interface ToastNotice {
  id: string;
  tone: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message?: string;
}

export interface NavEntry {
  page: PageId;
  instanceId?: string | null;
  tab?: InstanceDetailTab;
}

export interface LocalServerRecord {
  id: string;
  name: string;
  version: string;
  port: number;
  memory: number;
  eulaAccepted: boolean;
  javaPath?: string;
  state: 'stopped' | 'starting' | 'running' | 'failed';
  createdAt?: string;
  dir?: string;
}

export interface SolaraProfile {
  signedIn: boolean;
  displayName?: string | null;
  email?: string | null;
  handle?: string | null;
  avatarUrl?: string | null;
  role?: string | null;
  roles?: string[];
  tier?: string | null;
}

export interface SolaraServiceItem {
  id?: string;
  name?: string;
  title?: string;
  description?: string;
  summary?: string;
  status?: string;
  url?: string;
  href?: string;
  category?: string;
  badge?: string;
}

export interface SolaraBootstrapData {
  schemaVersion?: number;
  generatedAt?: string;
  events?: Array<Record<string, unknown>>;
  projects?: Array<Record<string, unknown>>;
  services?: SolaraServiceItem[];
  releases?: Array<Record<string, unknown>>;
  network?: {
    ip?: string;
    address?: string;
    name?: string;
    status?: string;
    version?: string;
    playersOnline?: number;
    maxPlayers?: number;
    [key: string]: unknown;
  };
  links?: Record<string, string>;
}

export interface LaunchProgressState {
  stage: 'checking' | 'java' | 'mods' | 'starting' | 'running' | 'error';
  detail: string;
  error?: string;
  updatedAt: number;
}

export type SettingsCategoryId =
  | 'appearance'
  | 'features'
  | 'behaviour'
  | 'accounts'
  | 'java'
  | 'window'
  | 'downloads'
  | 'storage'
  | 'privacy'
  | 'resources'
  | 'troubleshooting'
  | 'about';

export type InstanceSettingsCategoryId =
  | 'general'
  | 'installation'
  | 'java'
  | 'window'
  | 'hooks'
  | 'backups'
  | 'advanced';
