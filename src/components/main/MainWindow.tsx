import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { ReminderDueBanner } from "../ReminderDueBanner";
import { useAppStore } from "../../store/useAppStore";
import type { AppConfig, MainSection } from "../../types";
import { NotesView } from "./NotesView";
import { ProjectsView } from "./ProjectsView";
import { SettingsView } from "./SettingsView";
import { Sidebar } from "./Sidebar";

export function MainWindow() {
  const { ready, load, section, setSection, applyConfig, refreshNotes } =
    useAppStore();

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const unlisteners: Array<Promise<() => void>> = [];
    unlisteners.push(
      listen<AppConfig>("config-updated", (e) => {
        applyConfig(e.payload);
      }),
    );
    unlisteners.push(
      listen("notes-changed", () => {
        void refreshNotes();
      }),
    );
    unlisteners.push(
      listen<string>("navigate-section", (e) => {
        if (e.payload === "notes" || e.payload === "projects" || e.payload === "settings") {
          setSection(e.payload);
        }
      }),
    );
    return () => {
      void Promise.all(unlisteners).then((fns) => fns.forEach((fn) => fn()));
    };
  }, [applyConfig, refreshNotes, setSection]);

  if (!ready) {
    return (
      <div className="flex h-full items-center justify-center bg-surface text-ink-muted">
        Загрузка…
      </div>
    );
  }

  return (
    <div className="relative flex h-full bg-surface text-ink">
      <ReminderDueBanner onChanged={() => void refreshNotes()} />
      <Sidebar section={section} onChange={(s: MainSection) => setSection(s)} />
      <main className="flex min-w-0 flex-1 flex-col">
        {section === "notes" && <NotesView />}
        {section === "projects" && <ProjectsView />}
        {section === "settings" && <SettingsView />}
      </main>
    </div>
  );
}
