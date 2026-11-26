export type Theme = "dark" | "light" | "system";

export type RemindersMode = "off" | "relative" | "absolute" | "both";
export type ReminderKind = "relative" | "absolute";
export type ReminderStatus = "pending" | "done" | "snoozed";

export interface NoteReminder {
  kind: ReminderKind;
  /** Local `YYYY-MM-DD HH:mm`. */
  dueAt: string;
  relativeDays?: number | null;
  status: ReminderStatus;
  snoozeUntil?: string | null;
  notifiedAt?: string | null;
}

export interface ReminderInput {
  kind: ReminderKind;
  relativeDays?: number | null;
  /** Local `YYYY-MM-DD HH:mm` or `YYYY-MM-DDTHH:mm`. */
  absoluteAt?: string | null;
}

export interface ReminderDueEvent {
  projectId: string;
  noteId: string;
  projectName: string;
  bodyPreview: string;
  dueAt: string;
  kind: ReminderKind;
}

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
  /** Off / relative only / absolute only / both. */
  remindersMode: RemindersMode;
  /** OS toast / tray notifications when a reminder is due. */
  reminderNotifications: boolean;
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
  reminder?: NoteReminder | null;
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
