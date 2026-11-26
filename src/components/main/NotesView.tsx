import { Copy, ExternalLink, Pencil, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import { openPath } from "@tauri-apps/plugin-opener";
import { api } from "../../lib/api";
import { groupNotesByDay, highlightText } from "../../lib/notesUi";
import { useAppStore } from "../../store/useAppStore";
import type { Note, Project } from "../../types";

export function NotesView() {
  const {
    config,
    notes,
    selectedProjectId,
    setSelectedProjectId,
    searchQuery,
    setSearchQuery,
    searchAllProjects,
    setSearchAllProjects,
    deleteNoteWithUndo,
    undo,
    undoDelete,
    updateNote,
  } = useAppStore();

  const projects = useMemo(
    () => [...(config?.projects ?? [])].sort((a, b) => a.order - b.order),
    [config],
  );

  const noteCounts = useMemo(() => {
    const map = new Map<string, number>();
    for (const n of notes) {
      map.set(n.projectId, (map.get(n.projectId) ?? 0) + 1);
    }
    return map;
  }, [notes]);

  const filtered = useMemo(() => {
    let list = notes;
    if (!searchAllProjects && selectedProjectId) {
      list = list.filter((n) => n.projectId === selectedProjectId);
    }
    const q = searchQuery.trim().toLowerCase();
    if (q) list = list.filter((n) => n.body.toLowerCase().includes(q));
    return list;
  }, [notes, searchQuery, searchAllProjects, selectedProjectId]);

  const groups = groupNotesByDay(filtered);
  const selected = projects.find((p) => p.id === selectedProjectId) ?? null;

  return (
    <div className="flex min-h-0 flex-1">
      <div className="flex w-64 shrink-0 flex-col border-r border-line bg-surface/80">
        <div className="border-b border-line px-3 py-3 text-xs font-medium uppercase tracking-wide text-ink-faint">
          Проекты
        </div>
        <div className="flex-1 overflow-auto p-2">
          {projects.map((p) => (
            <ProjectRow
              key={p.id}
              project={p}
              active={p.id === selectedProjectId}
              count={noteCounts.get(p.id) ?? 0}
              onClick={() => setSelectedProjectId(p.id)}
            />
          ))}
          {!projects.length && (
            <div className="px-2 py-6 text-center text-sm text-ink-muted">
              Добавьте проект во вкладке «Проекты»
            </div>
          )}
        </div>
      </div>

      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex items-center gap-3 border-b border-line px-4 py-3">
          <input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Поиск по тексту…"
            className="min-w-0 flex-1 rounded-xl border border-line bg-surface-muted px-3 py-2 text-sm outline-none focus:border-accent/50"
          />
          <label className="flex items-center gap-2 text-xs text-ink-muted">
            <input
              type="checkbox"
              checked={searchAllProjects}
              onChange={(e) => setSearchAllProjects(e.target.checked)}
            />
            Все проекты
          </label>
          {selected && (
            <button
              type="button"
              title="Открыть папку проекта"
              className="rounded-lg border border-line p-2 text-ink-muted hover:bg-surface-muted hover:text-ink"
              onClick={async () => {
                const path = await api.getProjectFolderPath(selected.id);
                await openPath(path);
              }}
            >
              <ExternalLink className="h-4 w-4" />
            </button>
          )}
        </div>

        <div className="relative flex-1 overflow-auto px-4 py-4">
          {groups.map((g) => (
            <section key={g.label} className="mb-6">
              <h3 className="mb-3 text-xs font-semibold uppercase tracking-wide text-ink-faint">
                {g.label}
              </h3>
              <div className="space-y-3">
                {g.notes.map((note) => (
                  <NoteCard
                    key={note.id}
                    note={note}
                    project={projects.find((p) => p.id === note.projectId)}
                    query={searchQuery}
                    onDelete={() => void deleteNoteWithUndo(note)}
                    onSave={(text) => void updateNote(note, text)}
                  />
                ))}
              </div>
            </section>
          ))}
          {!filtered.length && (
            <div className="py-16 text-center text-sm text-ink-muted">
              {selected ? "Пока нет заметок" : "Выберите проект"}
            </div>
          )}

          {undo && (
            <div className="sticky bottom-2 flex justify-center">
              <div className="flex items-center gap-3 rounded-full border border-line bg-surface-elevated px-4 py-2 text-sm shadow-lg">
                <span>Заметка удалена</span>
                <button
                  type="button"
                  className="font-medium text-accent"
                  onClick={() => void undoDelete()}
                >
                  Отменить
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ProjectRow({
  project,
  active,
  count,
  onClick,
}: {
  project: Project;
  active: boolean;
  count: number;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`mb-1 flex w-full items-center gap-2 rounded-xl px-2.5 py-2 text-left text-sm ${
        active ? "bg-accent/15" : "hover:bg-surface-muted"
      }`}
    >
      <span
        className="h-2.5 w-2.5 rounded-full"
        style={{ background: project.color }}
      />
      <span>{project.emoji}</span>
      <span className="min-w-0 flex-1 truncate">{project.name}</span>
      <span className="text-xs text-ink-faint">{count}</span>
    </button>
  );
}

function NoteCard({
  note,
  project,
  query,
  onDelete,
  onSave,
}: {
  note: Note;
  project?: Project;
  query: string;
  onDelete: () => void;
  onSave: (text: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(note.body);
  const [confirmDelete, setConfirmDelete] = useState(false);

  return (
    <article className="rounded-2xl border border-line bg-surface-elevated p-4">
      <div className="mb-2 flex items-center gap-2 text-xs text-ink-faint">
        {project && (
          <>
            <span
              className="h-2 w-2 rounded-full"
              style={{ background: project.color }}
            />
            <span>
              {project.emoji} {project.name}
            </span>
            <span>·</span>
          </>
        )}
        <span>{note.timestamp}</span>
        <div className="ml-auto flex items-center gap-1">
          <IconBtn
            title="Копировать"
            onClick={() => void navigator.clipboard.writeText(note.body)}
          >
            <Copy className="h-3.5 w-3.5" />
          </IconBtn>
          <IconBtn
            title="Редактировать"
            onClick={() => {
              setDraft(note.body);
              setEditing(true);
            }}
          >
            <Pencil className="h-3.5 w-3.5" />
          </IconBtn>
          <IconBtn
            title="Удалить"
            onClick={() => setConfirmDelete(true)}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </IconBtn>
        </div>
      </div>

      {editing ? (
        <div>
          <textarea
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            className="min-h-24 w-full resize-y rounded-xl border border-line bg-surface-muted p-3 text-sm outline-none"
          />
          <div className="mt-2 flex justify-end gap-2">
            <button
              type="button"
              className="rounded-lg px-3 py-1.5 text-sm text-ink-muted hover:bg-surface-muted"
              onClick={() => setEditing(false)}
            >
              Отмена
            </button>
            <button
              type="button"
              className="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white"
              onClick={() => {
                onSave(draft);
                setEditing(false);
              }}
            >
              Сохранить
            </button>
          </div>
        </div>
      ) : (
        <div
          className="whitespace-pre-wrap text-sm leading-relaxed text-ink"
          dangerouslySetInnerHTML={{
            __html: highlightText(note.body, query),
          }}
        />
      )}

      {confirmDelete && (
        <div className="mt-3 flex items-center justify-between rounded-xl bg-rose-500/10 px-3 py-2 text-sm">
          <span>Удалить заметку?</span>
          <div className="flex gap-2">
            <button
              type="button"
              className="rounded-lg px-2 py-1 text-ink-muted hover:bg-surface-muted"
              onClick={() => setConfirmDelete(false)}
            >
              Нет
            </button>
            <button
              type="button"
              className="rounded-lg bg-rose-500 px-2 py-1 text-white"
              onClick={() => {
                setConfirmDelete(false);
                onDelete();
              }}
            >
              Удалить
            </button>
          </div>
        </div>
      )}
    </article>
  );
}

function IconBtn({
  children,
  title,
  onClick,
}: {
  children: React.ReactNode;
  title: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      title={title}
      onClick={onClick}
      className="rounded-md p-1.5 text-ink-faint hover:bg-surface-muted hover:text-ink"
    >
      {children}
    </button>
  );
}
