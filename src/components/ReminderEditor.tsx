import type { ReminderInput, ReminderKind, RemindersMode } from "../types";

export type ReminderDraft = {
  enabled: boolean;
  kind: ReminderKind;
  relativeDays: number;
  absoluteAt: string;
};

export function emptyReminderDraft(
  mode: RemindersMode,
  existing?: ReminderInput | null,
): ReminderDraft {
  const kind: ReminderKind =
    mode === "absolute"
      ? "absolute"
      : mode === "relative"
        ? "relative"
        : (existing?.kind ?? "relative");
  return {
    enabled: Boolean(existing),
    kind,
    relativeDays: existing?.relativeDays ?? 1,
    absoluteAt:
      existing?.absoluteAt ??
      defaultAbsoluteLocal(),
  };
}

export function draftFromNoteReminder(
  mode: RemindersMode,
  reminder: {
    kind: ReminderKind;
    dueAt: string;
    relativeDays?: number | null;
    status?: string;
  } | null | undefined,
): ReminderDraft {
  if (!reminder || reminder.status === "done") {
    return emptyReminderDraft(mode, null);
  }
  return {
    enabled: true,
    kind: reminder.kind,
    relativeDays: reminder.relativeDays ?? 1,
    absoluteAt: reminder.dueAt.replace(" ", "T"),
  };
}

function defaultAbsoluteLocal(): string {
  const d = new Date();
  d.setMinutes(0, 0, 0);
  d.setHours(d.getHours() + 1);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function reminderDraftToInput(
  draft: ReminderDraft,
): ReminderInput | null {
  if (!draft.enabled) return null;
  if (draft.kind === "relative") {
    return {
      kind: "relative",
      relativeDays: Math.max(1, Math.floor(draft.relativeDays) || 1),
    };
  }
  return {
    kind: "absolute",
    absoluteAt: draft.absoluteAt,
  };
}

export function ReminderEditor({
  mode,
  value,
  onChange,
  compact,
}: {
  mode: RemindersMode;
  value: ReminderDraft;
  onChange: (next: ReminderDraft) => void;
  compact?: boolean;
}) {
  if (mode === "off") return null;

  const allowRelative = mode === "relative" || mode === "both";
  const allowAbsolute = mode === "absolute" || mode === "both";
  const showKindToggle = allowRelative && allowAbsolute;

  return (
    <div
      className={
        compact
          ? "flex flex-wrap items-center gap-2 px-3 py-1.5 text-xs"
          : "space-y-2 rounded-xl border border-line bg-surface-muted/50 p-3 text-sm"
      }
    >
      <label className="flex items-center gap-2 text-ink-muted">
        <input
          type="checkbox"
          checked={value.enabled}
          onChange={(e) =>
            onChange({ ...value, enabled: e.target.checked })
          }
        />
        <span>Напоминание</span>
      </label>

      {value.enabled && (
        <>
          {showKindToggle && (
            <div className="flex gap-1">
              <KindBtn
                active={value.kind === "relative"}
                onClick={() => onChange({ ...value, kind: "relative" })}
                compact={compact}
              >
                Через N дней
              </KindBtn>
              <KindBtn
                active={value.kind === "absolute"}
                onClick={() => onChange({ ...value, kind: "absolute" })}
                compact={compact}
              >
                Дата/время
              </KindBtn>
            </div>
          )}

          {value.kind === "relative" && allowRelative && (
            <label className="flex items-center gap-2 text-ink-muted">
              <span>Через</span>
              <input
                type="number"
                min={1}
                max={3650}
                value={value.relativeDays}
                onChange={(e) =>
                  onChange({
                    ...value,
                    relativeDays: Number(e.target.value) || 1,
                  })
                }
                className="w-16 rounded-lg border border-line bg-surface-elevated px-2 py-1 text-ink outline-none"
              />
              <span>дн.</span>
            </label>
          )}

          {value.kind === "absolute" && allowAbsolute && (
            <input
              type="datetime-local"
              value={toDatetimeLocalValue(value.absoluteAt)}
              onChange={(e) =>
                onChange({ ...value, absoluteAt: e.target.value })
              }
              className="rounded-lg border border-line bg-surface-elevated px-2 py-1 text-ink outline-none"
            />
          )}
        </>
      )}
    </div>
  );
}

function toDatetimeLocalValue(raw: string): string {
  if (!raw) return "";
  const s = raw.replace(" ", "T");
  // datetime-local wants YYYY-MM-DDTHH:mm
  return s.length >= 16 ? s.slice(0, 16) : s;
}

function KindBtn({
  active,
  onClick,
  children,
  compact,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
  compact?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-lg px-2 py-1 ${compact ? "text-[11px]" : "text-xs"} ${
        active
          ? "bg-accent text-white"
          : "border border-line bg-surface-elevated text-ink-muted"
      }`}
    >
      {children}
    </button>
  );
}
