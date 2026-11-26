import { FolderKanban, NotebookPen, Settings } from "lucide-react";
import type { MainSection } from "../../types";

const items: { id: MainSection; label: string; icon: typeof NotebookPen }[] = [
  { id: "notes", label: "Заметки", icon: NotebookPen },
  { id: "projects", label: "Проекты", icon: FolderKanban },
  { id: "settings", label: "Настройки", icon: Settings },
];

export function Sidebar({
  section,
  onChange,
}: {
  section: MainSection;
  onChange: (s: MainSection) => void;
}) {
  return (
    <aside className="flex w-52 shrink-0 flex-col border-r border-line bg-surface-muted/60 px-3 py-4">
      <div className="mb-6 px-2">
        <div className="text-lg font-semibold tracking-tight text-ink">
          QuickNotes
        </div>
        <div className="text-xs text-ink-faint">Быстрые заметки по проектам</div>
      </div>
      <nav className="flex flex-col gap-1">
        {items.map(({ id, label, icon: Icon }) => {
          const active = section === id;
          return (
            <button
              key={id}
              type="button"
              onClick={() => onChange(id)}
              className={`flex items-center gap-2 rounded-xl px-3 py-2 text-sm transition ${
                active
                  ? "bg-accent/15 text-ink font-medium"
                  : "text-ink-muted hover:bg-surface-elevated hover:text-ink"
              }`}
            >
              <Icon className="h-4 w-4" />
              {label}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
