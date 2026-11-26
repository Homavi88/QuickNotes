export type Theme = "dark" | "light" | "system";

export interface Settings {
  hotkey: string;
  autostart: boolean;
  theme: Theme;
  accentColor: string;
  storageDir: string;
  hideOnBlur: boolean;
  glassEffect: boolean;
  lastProjectId: string | null;
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

export const PROJECT_EMOJIS = [
  "🚀",
  "💡",
  "📝",
  "🎯",
  "🔧",
  "📚",
  "🌟",
  "🧠",
  "📦",
  "🎨",
  "💼",
  "🔥",
] as const;
