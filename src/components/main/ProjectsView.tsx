import { Reorder } from "framer-motion";
import { GripVertical, Plus, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useAppStore } from "../../store/useAppStore";
import { PROJECT_COLORS, PROJECT_EMOJIS, type Project } from "../../types";

export function ProjectsView() {
  const { config, createProject, updateProject, deleteProject, reorderProjects } =
    useAppStore();
  const [items, setItems] = useState<Project[]>([]);
  const [name, setName] = useState("");
  const [color, setColor] = useState<string>(PROJECT_COLORS[0]);
  const [emoji, setEmoji] = useState<string>(PROJECT_EMOJIS[0]);
  const [deleteTarget, setDeleteTarget] = useState<Project | null>(null);

  useEffect(() => {
    setItems([...(config?.projects ?? [])].sort((a, b) => a.order - b.order));
  }, [config]);

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-auto p-6">
      <div className="mb-6">
        <h2 className="text-xl font-semibold">Проекты</h2>
        <p className="text-sm text-ink-muted">
          Цвет, эмодзи и порядок. Перетащите строку за ручку, чтобы изменить порядок.
        </p>
      </div>

      <div className="mb-6 rounded-2xl border border-line bg-surface-elevated p-4">
        <div className="mb-3 text-sm font-medium">Новый проект</div>
        <div className="flex flex-wrap items-end gap-3">
          <label className="flex min-w-[200px] flex-1 flex-col gap-1 text-xs text-ink-muted">
            Название
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="rounded-xl border border-line bg-surface-muted px-3 py-2 text-sm text-ink outline-none"
              placeholder="Например, Клиент X"
            />
          </label>
          <div className="flex flex-col gap-1 text-xs text-ink-muted">
            Цвет
            <div className="flex flex-wrap gap-1.5">
              {PROJECT_COLORS.map((c) => (
                <button
                  key={c}
                  type="button"
                  onClick={() => setColor(c)}
                  className={`h-6 w-6 rounded-full border-2 ${
                    color === c ? "border-ink" : "border-transparent"
                  }`}
                  style={{ background: c }}
                />
              ))}
            </div>
          </div>
          <div className="flex flex-col gap-1 text-xs text-ink-muted">
            Эмодзи
            <div className="flex flex-wrap gap-1">
              {PROJECT_EMOJIS.map((e) => (
                <button
                  key={e}
                  type="button"
                  onClick={() => setEmoji(e)}
                  className={`rounded-lg px-1.5 py-1 text-base ${
                    emoji === e ? "bg-accent/20" : "hover:bg-surface-muted"
                  }`}
                >
                  {e}
                </button>
              ))}
            </div>
          </div>
          <button
            type="button"
            disabled={!name.trim()}
            onClick={async () => {
              await createProject(name.trim(), color, emoji);
              setName("");
            }}
            className="inline-flex items-center gap-2 rounded-xl bg-accent px-4 py-2 text-sm font-medium text-white disabled:opacity-40"
          >
            <Plus className="h-4 w-4" />
            Добавить
          </button>
        </div>
      </div>

      <Reorder.Group
        axis="y"
        values={items}
        onReorder={setItems}
        onMouseUp={() => {
          void reorderProjects(items.map((p) => p.id));
        }}
        className="space-y-2"
      >
        {items.map((project) => (
          <Reorder.Item
            key={project.id}
            value={project}
            className="flex items-center gap-3 rounded-2xl border border-line bg-surface-elevated px-3 py-3"
          >
            <GripVertical className="h-4 w-4 cursor-grab text-ink-faint active:cursor-grabbing" />
            <span
              className="h-3 w-3 rounded-full"
              style={{ background: project.color }}
            />
            <select
              value={project.emoji}
              onChange={(e) =>
                void updateProject({ id: project.id, emoji: e.target.value })
              }
              className="rounded-lg bg-surface-muted px-1 py-1 text-base outline-none"
            >
              {PROJECT_EMOJIS.map((e) => (
                <option key={e} value={e}>
                  {e}
                </option>
              ))}
            </select>
            <input
              defaultValue={project.name}
              key={`${project.id}-${project.name}`}
              onBlur={(e) => {
                const next = e.target.value.trim();
                if (next && next !== project.name) {
                  void updateProject({ id: project.id, name: next });
                } else {
                  e.target.value = project.name;
                }
              }}
              className="min-w-0 flex-1 rounded-lg bg-transparent px-2 py-1 text-sm outline-none hover:bg-surface-muted focus:bg-surface-muted"
            />
            <div className="flex gap-1">
              {PROJECT_COLORS.map((c) => (
                <button
                  key={c}
                  type="button"
                  title={c}
                  onClick={() => void updateProject({ id: project.id, color: c })}
                  className={`h-4 w-4 rounded-full ${
                    project.color === c ? "ring-2 ring-ink/40 ring-offset-1" : ""
                  }`}
                  style={{ background: c }}
                />
              ))}
            </div>
            <button
              type="button"
              className="rounded-lg p-2 text-ink-faint hover:bg-rose-500/10 hover:text-rose-400"
              onClick={() => setDeleteTarget(project)}
            >
              <Trash2 className="h-4 w-4" />
            </button>
          </Reorder.Item>
        ))}
      </Reorder.Group>

      {deleteTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
          <div className="w-full max-w-md rounded-2xl border border-line bg-surface-elevated p-5 shadow-xl">
            <h3 className="text-lg font-semibold">Удалить проект?</h3>
            <p className="mt-2 text-sm text-ink-muted">
              «{deleteTarget.name}» будет удалён из списка. Что сделать с папкой на
              диске?
            </p>
            <div className="mt-5 flex flex-wrap justify-end gap-2">
              <button
                type="button"
                className="rounded-xl px-3 py-2 text-sm text-ink-muted hover:bg-surface-muted"
                onClick={() => setDeleteTarget(null)}
              >
                Отмена
              </button>
              <button
                type="button"
                className="rounded-xl border border-line px-3 py-2 text-sm hover:bg-surface-muted"
                onClick={async () => {
                  await deleteProject(deleteTarget.id, false);
                  setDeleteTarget(null);
                }}
              >
                Оставить файлы
              </button>
              <button
                type="button"
                className="rounded-xl bg-rose-500 px-3 py-2 text-sm font-medium text-white"
                onClick={async () => {
                  await deleteProject(deleteTarget.id, true);
                  setDeleteTarget(null);
                }}
              >
                Удалить файлы
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
