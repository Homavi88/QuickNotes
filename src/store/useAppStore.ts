import { create } from "zustand";
import { api } from "../lib/api";
import { applyTheme } from "../lib/theme";
import type {
  AppConfig,
  MainSection,
  Note,
  NotesFilter,
  Project,
  Settings,
} from "../types";

interface AppState {
  ready: boolean;
  config: AppConfig | null;
  notes: Note[];
  selectedProjectId: string | null;
  searchQuery: string;
  searchAllProjects: boolean;
  notesFilter: NotesFilter;
  section: MainSection;
  undo: { note: Note; timer: number } | null;
  load: () => Promise<void>;
  refreshNotes: () => Promise<void>;
  setSection: (section: MainSection) => void;
  setSelectedProjectId: (id: string | null) => void;
  setSearchQuery: (q: string) => void;
  setSearchAllProjects: (v: boolean) => void;
  setNotesFilter: (filter: NotesFilter) => void;
  applyConfig: (config: AppConfig) => void;
  updateSettings: (settings: Settings) => Promise<void>;
  createProject: (name: string, color: string, emoji: string) => Promise<Project>;
  updateProject: (input: {
    id: string;
    name?: string;
    color?: string;
    emoji?: string;
  }) => Promise<void>;
  deleteProject: (id: string, deleteFiles: boolean) => Promise<void>;
  reorderProjects: (orderedIds: string[]) => Promise<void>;
  deleteNoteWithUndo: (note: Note) => Promise<void>;
  undoDelete: () => Promise<void>;
  updateNote: (
    note: Note,
    text: string,
    opts?: {
      reminder?: import("../types").ReminderInput | null;
      setReminder?: boolean;
    },
  ) => Promise<void>;
  setNoteClosed: (note: Note, closed: boolean) => Promise<void>;
  completeReminder: (note: Note) => Promise<void>;
  snoozeReminder: (
    note: Note,
    opts?: { minutes?: number; until?: string },
  ) => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  ready: false,
  config: null,
  notes: [],
  selectedProjectId: null,
  searchQuery: "",
  searchAllProjects: false,
  notesFilter: "active",
  section: "notes",
  undo: null,

  load: async () => {
    const config = await api.getConfig();
    applyTheme(config.settings.theme, config.settings.accentColor);
    const selected =
      config.settings.lastProjectId ?? config.projects[0]?.id ?? null;
    set({
      config,
      selectedProjectId: selected,
      ready: true,
    });
    await get().refreshNotes();
  },

  refreshNotes: async () => {
    // Always load all notes so sidebar counts stay accurate.
    const notes = await api.listNotes(null);
    set({ notes });
  },

  setSection: (section) => set({ section }),
  setSelectedProjectId: (id) => {
    set({ selectedProjectId: id });
    void api.setLastProject(id);
  },
  setSearchQuery: (q) => set({ searchQuery: q }),
  setSearchAllProjects: (v) => set({ searchAllProjects: v }),
  setNotesFilter: (notesFilter) => set({ notesFilter }),

  applyConfig: (config) => {
    applyTheme(config.settings.theme, config.settings.accentColor);
    set((state) => ({
      config,
      selectedProjectId:
        state.selectedProjectId &&
        config.projects.some((p) => p.id === state.selectedProjectId)
          ? state.selectedProjectId
          : (config.settings.lastProjectId ?? config.projects[0]?.id ?? null),
    }));
  },

  updateSettings: async (settings) => {
    const config = await api.updateSettings(settings);
    get().applyConfig(config);
    await api.applyGlassEffect(settings.glassEffect);
  },

  createProject: async (name, color, emoji) => {
    const project = await api.createProject({ name, color, emoji });
    const config = await api.getConfig();
    get().applyConfig(config);
    set({ selectedProjectId: project.id });
    await get().refreshNotes();
    return project;
  },

  updateProject: async (input) => {
    await api.updateProject(input);
    const config = await api.getConfig();
    get().applyConfig(config);
  },

  deleteProject: async (id, deleteFiles) => {
    await api.deleteProject(id, deleteFiles);
    const config = await api.getConfig();
    get().applyConfig(config);
    await get().refreshNotes();
  },

  reorderProjects: async (orderedIds) => {
    await api.reorderProjects(orderedIds);
    const config = await api.getConfig();
    get().applyConfig(config);
  },

  deleteNoteWithUndo: async (note) => {
    const prev = get().undo;
    if (prev) window.clearTimeout(prev.timer);
    await api.deleteNote(note.projectId, note.id);
    await get().refreshNotes();
    const timer = window.setTimeout(() => set({ undo: null }), 5000);
    set({ undo: { note, timer } });
  },

  undoDelete: async () => {
    const undo = get().undo;
    if (!undo) return;
    window.clearTimeout(undo.timer);
    const rem = undo.note.reminder;
    await api.saveNote(
      undo.note.projectId,
      undo.note.body,
      null,
      rem && rem.status !== "done" ? rem : null,
      {
        closed: !!undo.note.closed,
        closedAt: undo.note.closedAt ?? null,
      },
    );
    set({ undo: null });
    await get().refreshNotes();
  },

  updateNote: async (note, text, opts) => {
    await api.updateNote(note.projectId, note.id, text, opts);
    await get().refreshNotes();
  },

  setNoteClosed: async (note, closed) => {
    await api.setNoteClosed(note.projectId, note.id, closed);
    await get().refreshNotes();
  },

  completeReminder: async (note) => {
    await api.completeReminder(note.projectId, note.id);
    await get().refreshNotes();
  },

  snoozeReminder: async (note, opts) => {
    await api.snoozeReminder(note.projectId, note.id, opts);
    await get().refreshNotes();
  },
}));
