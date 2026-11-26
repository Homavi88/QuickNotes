import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  Draft,
  Note,
  Project,
  Settings,
} from "../types";

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  updateSettings: (settings: Settings) =>
    invoke<AppConfig>("update_settings", { settings }),
  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (input: { name: string; color: string; emoji: string }) =>
    invoke<Project>("create_project", { input }),
  updateProject: (input: {
    id: string;
    name?: string;
    color?: string;
    emoji?: string;
  }) => invoke<Project>("update_project", { input }),
  deleteProject: (id: string, deleteFiles: boolean) =>
    invoke<void>("delete_project", { input: { id, deleteFiles } }),
  reorderProjects: (orderedIds: string[]) =>
    invoke<Project[]>("reorder_projects", { input: { orderedIds } }),
  listNotes: (projectId?: string | null) =>
    invoke<Note[]>("list_notes", { projectId: projectId ?? null }),
  saveNote: (projectId: string, text: string) =>
    invoke<Note>("save_note", { input: { projectId, text } }),
  updateNote: (projectId: string, noteId: string, text: string) =>
    invoke<Note>("update_note", { input: { projectId, noteId, text } }),
  deleteNote: (projectId: string, noteId: string) =>
    invoke<Note>("delete_note", { input: { projectId, noteId } }),
  saveDraft: (draft: Draft | null) =>
    invoke<void>("save_draft", { draft }),
  setLastProject: (projectId: string | null) =>
    invoke<void>("set_last_project", { projectId }),
  getProjectFolderPath: (projectId: string) =>
    invoke<string>("get_project_folder_path", { projectId }),
  getStorageDir: () => invoke<string>("get_storage_dir"),
  showQuickWindow: () => invoke<void>("show_quick_window"),
  hideQuickWindow: () => invoke<void>("hide_quick_window"),
  showMainWindow: (section?: string) =>
    invoke<void>("show_main_window", { section: section ?? null }),
  applyGlassEffect: (enabled: boolean) =>
    invoke<void>("apply_glass_effect", { enabled }),
};
