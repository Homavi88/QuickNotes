import { useMemo, useState } from "react";
import {
  EMOJI_CATEGORIES,
  PROJECT_EMOJIS,
  type EmojiCategoryId,
} from "../lib/emojis";

export function EmojiPicker({
  value,
  onChange,
  compact = false,
}: {
  value: string;
  onChange: (emoji: string) => void;
  /** Smaller popover for inline row edit. */
  compact?: boolean;
}) {
  const [open, setOpen] = useState(!compact);
  const [category, setCategory] = useState<EmojiCategoryId>("favorites");
  const [showAll, setShowAll] = useState(false);

  const list = useMemo(() => {
    if (showAll) return PROJECT_EMOJIS;
    const cat = EMOJI_CATEGORIES.find((c) => c.id === category);
    return [...(cat?.emojis ?? PROJECT_EMOJI_FAVORITES_FALLBACK)];
  }, [category, showAll]);

  const panel = (
    <div
      className={`overflow-hidden rounded-xl border border-line bg-surface-elevated shadow-lg ${
        compact ? "w-80" : "w-full max-w-lg"
      }`}
    >
      <div className="flex flex-wrap gap-1 border-b border-line/60 p-2">
        <button
          type="button"
          onClick={() => {
            setShowAll(true);
          }}
          className={`rounded-lg px-2 py-1 text-[11px] ${
            showAll
              ? "bg-accent text-white"
              : "bg-surface-muted text-ink-muted hover:text-ink"
          }`}
        >
          Все ({PROJECT_EMOJIS.length})
        </button>
        {EMOJI_CATEGORIES.map((c) => (
          <button
            key={c.id}
            type="button"
            onClick={() => {
              setCategory(c.id);
              setShowAll(false);
            }}
            className={`rounded-lg px-2 py-1 text-[11px] ${
              !showAll && category === c.id
                ? "bg-accent text-white"
                : "bg-surface-muted text-ink-muted hover:text-ink"
            }`}
          >
            {c.label}
          </button>
        ))}
      </div>
      <div className="max-h-52 overflow-auto p-2">
        <div className="grid grid-cols-8 gap-0.5 sm:grid-cols-10">
          {list.map((e) => (
            <button
              key={`${showAll ? "all" : category}-${e}`}
              type="button"
              title={e}
              onClick={() => {
                onChange(e);
                if (compact) setOpen(false);
              }}
              className={`rounded-lg px-1 py-1.5 text-lg leading-none hover:bg-surface-muted ${
                value === e ? "bg-accent/20 ring-1 ring-accent/40" : ""
              }`}
            >
              {e}
            </button>
          ))}
        </div>
      </div>
    </div>
  );

  if (compact) {
    return (
      <div className="relative">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          className="rounded-lg bg-surface-muted px-2 py-1 text-base outline-none hover:bg-surface-muted/80"
          title="Выбрать эмодзи"
        >
          {value}
        </button>
        {open && (
          <>
            <button
              type="button"
              className="fixed inset-0 z-30 cursor-default"
              aria-label="Закрыть"
              onClick={() => setOpen(false)}
            />
            <div className="absolute left-0 top-full z-40 mt-1">{panel}</div>
          </>
        )}
      </div>
    );
  }

  return (
    <div className="flex min-w-[220px] flex-1 flex-col gap-1 text-xs text-ink-muted">
      <div className="flex items-center gap-2">
        <span>Эмодзи</span>
        <span className="text-lg leading-none text-ink">{value}</span>
      </div>
      {panel}
    </div>
  );
}

const PROJECT_EMOJI_FAVORITES_FALLBACK = ["📝"] as const;
