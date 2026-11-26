import { Bell, Check, Clock } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "../lib/api";
import type { ReminderDueEvent } from "../types";

export function ReminderDueBanner({
  onChanged,
}: {
  onChanged?: () => void;
}) {
  const [queue, setQueue] = useState<ReminderDueEvent[]>([]);
  const current = queue[0] ?? null;

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    void listen<ReminderDueEvent>("reminder-due", (e) => {
      setQueue((q) => {
        if (q.some((x) => x.noteId === e.payload.noteId)) return q;
        return [...q, e.payload];
      });
    }).then((u) => unsubs.push(u));
    void listen<{ projectId: string; noteId: string }>(
      "reminder-resolved",
      (e) => {
        setQueue((q) => q.filter((x) => x.noteId !== e.payload.noteId));
      },
    ).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, []);

  const dismissLocal = (noteId: string) =>
    setQueue((q) => q.filter((x) => x.noteId !== noteId));

  const complete = async () => {
    if (!current) return;
    try {
      await api.completeReminder(current.projectId, current.noteId);
      dismissLocal(current.noteId);
      onChanged?.();
    } catch (e) {
      console.error(e);
    }
  };

  const snooze = async (minutes?: number) => {
    if (!current) return;
    try {
      await api.snoozeReminder(current.projectId, current.noteId, {
        minutes,
      });
      dismissLocal(current.noteId);
      onChanged?.();
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <AnimatePresence>
      {current && (
        <motion.div
          key={current.noteId}
          initial={{ opacity: 0, y: -12 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          className="pointer-events-auto fixed inset-x-0 top-3 z-50 flex justify-center px-4"
        >
          <div className="w-full max-w-lg rounded-2xl border border-amber-500/40 bg-surface-elevated p-4 shadow-lg">
            <div className="mb-2 flex items-start gap-2">
              <Bell className="mt-0.5 h-4 w-4 shrink-0 text-amber-400" />
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium text-ink">
                  Напоминание · {current.projectName}
                </div>
                <div className="mt-0.5 text-xs text-ink-faint">
                  к {current.dueAt}
                </div>
                <p className="mt-2 line-clamp-3 text-sm text-ink-muted">
                  {current.bodyPreview || "(пустая заметка)"}
                </p>
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <button
                type="button"
                onClick={() => void complete()}
                className="inline-flex items-center gap-1 rounded-lg bg-emerald-500/90 px-3 py-1.5 text-xs font-medium text-white"
              >
                <Check className="h-3.5 w-3.5" />
                Готово
              </button>
              <button
                type="button"
                onClick={() => void snooze(15)}
                className="inline-flex items-center gap-1 rounded-lg border border-line px-3 py-1.5 text-xs text-ink-muted hover:bg-surface-muted"
              >
                <Clock className="h-3.5 w-3.5" />
                +15 мин
              </button>
              <button
                type="button"
                onClick={() => void snooze(60)}
                className="rounded-lg border border-line px-3 py-1.5 text-xs text-ink-muted hover:bg-surface-muted"
              >
                +1 час
              </button>
              <button
                type="button"
                onClick={() => void snooze(undefined)}
                className="rounded-lg border border-line px-3 py-1.5 text-xs text-ink-muted hover:bg-surface-muted"
              >
                Завтра 09:00
              </button>
              <button
                type="button"
                onClick={() => dismissLocal(current.noteId)}
                className="ml-auto text-xs text-ink-faint hover:text-ink-muted"
              >
                Скрыть
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
