import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  Draft,
  Note,
  Project,
  RelocateDataRootResult,
  ReminderInput,
  RuntimePaths,
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
  saveNote: (
    projectId: string,
    text: string,
    reminder?: ReminderInput | null,
    reminderSnapshot?: import("../types").NoteReminder | null,
    closed?: { closed?: boolean; closedAt?: string | null },
  ) =>
    invoke<Note>("save_note", {
      input: {
        projectId,
        text,
        reminder: reminder ?? null,
        reminderSnapshot: reminderSnapshot ?? null,
        closed: closed?.closed ?? false,
        closedAt: closed?.closedAt ?? null,
      },
    }),
  setNoteClosed: (projectId: string, noteId: string, closed: boolean) =>
    invoke<Note>("set_note_closed", {
      input: { projectId, noteId, closed },
    }),
  updateNote: (
    projectId: string,
    noteId: string,
    text: string,
    opts?: { reminder?: ReminderInput | null; setReminder?: boolean },
  ) =>
    invoke<Note>("update_note", {
      input: {
        projectId,
        noteId,
        text,
        reminder: opts?.reminder ?? null,
        setReminder: opts?.setReminder ?? false,
      },
    }),
  deleteNote: (projectId: string, noteId: string) =>
    invoke<Note>("delete_note", { input: { projectId, noteId } }),
  completeReminder: (projectId: string, noteId: string) =>
    invoke<Note>("complete_reminder", { input: { projectId, noteId } }),
  snoozeReminder: (
    projectId: string,
    noteId: string,
    opts?: { minutes?: number; until?: string },
  ) =>
    invoke<Note>("snooze_reminder", {
      input: {
        projectId,
        noteId,
        minutes: opts?.minutes ?? null,
        until: opts?.until ?? null,
      },
    }),
  saveDraft: (draft: Draft | null) =>
    invoke<void>("save_draft", { draft }),
  setLastProject: (projectId: string | null) =>
    invoke<void>("set_last_project", { projectId }),
  getProjectFolderPath: (projectId: string) =>
    invoke<string>("get_project_folder_path", { projectId }),
  getStorageDir: () => invoke<string>("get_storage_dir"),
  getRuntimePaths: () => invoke<RuntimePaths>("get_runtime_paths"),
  relocateDataRoot: (parentPath: string, copyExisting: boolean) =>
    invoke<RelocateDataRootResult>("relocate_data_root", {
      input: { parentPath, copyExisting },
    }),
  resetDataRoot: (copyBack: boolean) =>
    invoke<RelocateDataRootResult>("reset_data_root", {
      input: { copyBack },
    }),
  showQuickWindow: () => invoke<void>("show_quick_window"),
  hideQuickWindow: () => invoke<void>("hide_quick_window"),
  showMainWindow: (section?: string) =>
    invoke<void>("show_main_window", { section: section ?? null }),
  applyGlassEffect: (enabled: boolean) =>
    invoke<void>("apply_glass_effect", { enabled }),
};
