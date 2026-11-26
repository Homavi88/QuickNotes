export type Theme = "dark" | "light" | "system";

export interface Settings {
  hotkey: string;
  /** In-window shortcut to save the quick note (default Ctrl+Enter). */
  saveHotkey: string;
  autostart: boolean;
  theme: Theme;
  accentColor: string;
  storageDir: string;
  hideOnBlur: boolean;
  /** Save non-empty text to the selected project on Esc / blur / hide. */
  saveOnHide: boolean;
  glassEffect: boolean;
  lastProjectId: string | null;
}

/** Where config/notes live (portable folder vs installed AppData). */
export interface RuntimePaths {
  portable: boolean;
  exeDir: string;
  configDir: string;
  configPath: string;
  defaultStorageDir: string;
  /** User-chosen parent when data relocated (`{parent}/QuickNotes/data`). */
  dataParent: string | null;
  defaultDataParent: string;
  /** Always `QuickNotes/data` under the chosen parent. */
  dataDirPreviewSuffix: string;
}

export interface Project {
  id: string;
  name: string;
  folderName: string;
  color: string;
  emoji: string;
  order: number;
}

export interface Draft {
  text: string;
  projectId: string | null;
}

export interface AppConfig {
  version: number;
  settings: Settings;
  projects: Project[];
  draft?: Draft | null;
}

export interface RelocateDataRootResult {
  config: AppConfig;
  paths: RuntimePaths;
  migrated: boolean;
}

export interface Note {
  id: string;
  projectId: string;
  timestamp: string;
  body: string;
}

export type MainSection = "notes" | "projects" | "settings";

export const PROJECT_COLORS = [
  "#FF6B6B",
  "#FF8E53",
  "#FFD166",
  "#06D6A0",
  "#118AB2",
  "#7C5CFF",
  "#EF476F",
  "#4ECDC4",
  "#A78BFA",
  "#F4A261",
  "#2A9D8F",
  "#E76F51",
] as const;

export { PROJECT_EMOJIS, EMOJI_CATEGORIES, PROJECT_EMOJI_FAVORITES } from "./lib/emojis";
