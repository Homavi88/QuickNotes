import { AnimatePresence, motion } from "framer-motion";
import { Check, ChevronDown, Search } from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  ReminderEditor,
  emptyReminderDraft,
  reminderDraftToInput,
  type ReminderDraft,
} from "../ReminderEditor";
import { api } from "../../lib/api";
import { formatHotkeyLabel, matchesHotkey } from "../../lib/hotkey";
import { applyTheme } from "../../lib/theme";
import type { AppConfig, Project, RemindersMode } from "../../types";

export function QuickWindow() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [text, setText] = useState("");
  const [projectId, setProjectId] = useState<string | null>(null);
  const [openList, setOpenList] = useState(false);
  const [projectQuery, setProjectQuery] = useState("");
  const [saving, setSaving] = useState(false);
  const [savedFlash, setSavedFlash] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [visible, setVisible] = useState(false);
  const [reminder, setReminder] = useState<ReminderDraft>(
    emptyReminderDraft("both"),
  );
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const draftTimer = useRef<number | null>(null);
  const hidingRef = useRef(false);
  const textRef = useRef(text);
  const selectedIdRef = useRef<string | null>(null);
  const reminderRef = useRef(reminder);

  const projects = useMemo(() => {
    const list = [...(config?.projects ?? [])].sort((a, b) => a.order - b.order);
    if (!projectQuery.trim()) return list;
    const q = projectQuery.toLowerCase();
    return list.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.emoji.includes(q),
    );
  }, [config, projectQuery]);

  const selected = projects.find((p) => p.id === projectId) ?? projects[0] ?? null;
  const saveHotkey = formatHotkeyLabel(
    config?.settings.saveHotkey ?? "Ctrl+Enter",
  );
  const saveOnHide = config?.settings.saveOnHide ?? false;
  const remindersMode: RemindersMode =
    config?.settings.remindersMode ?? "both";

  textRef.current = text;
  selectedIdRef.current = selected?.id ?? null;
  reminderRef.current = reminder;

  const load = useCallback(async () => {
    const cfg = await api.getConfig();
    setConfig(cfg);
    applyTheme(cfg.settings.theme, cfg.settings.accentColor);
    const nextProject =
      cfg.draft?.projectId ??
      cfg.settings.lastProjectId ??
      cfg.projects[0]?.id ??
      null;
    setProjectId(nextProject);
    setText(cfg.draft?.text ?? "");
    setReminder(emptyReminderDraft(cfg.settings.remindersMode ?? "both"));
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const persistDraft = useCallback(
    async (value: string, pid: string | null) => {
      await api.saveDraft(
        value.trim()
          ? { text: value, projectId: pid }
          : null,
      );
    },
    [],
  );

  const scheduleDraft = (value: string, pid: string | null) => {
    if (draftTimer.current) window.clearTimeout(draftTimer.current);
    draftTimer.current = window.setTimeout(() => {
      void persistDraft(value, pid);
    }, 250);
  };

  const hide = useCallback(async (saveDraftNow: boolean) => {
    if (hidingRef.current) return;
    hidingRef.current = true;
    if (saveDraftNow) {
      await persistDraft(textRef.current, selectedIdRef.current);
    }
    setVisible(false);
    window.setTimeout(() => {
      void api.hideQuickWindow();
    }, 160);
  }, [persistDraft]);

  const save = useCallback(
    async (opts?: { flash?: boolean }) => {
      const flash = opts?.flash ?? true;
      const pid = selectedIdRef.current;
      const value = textRef.current;
      if (!pid) {
        setError("Создайте проект в главном окне");
        return false;
      }
      if (!value.trim()) {
        setError("Пустая заметка");
        return false;
      }
      setSaving(true);
      setError(null);
      try {
        if (draftTimer.current) window.clearTimeout(draftTimer.current);
        const remInput = reminderDraftToInput(reminderRef.current);
        await api.saveNote(pid, value, remInput);
        setText("");
        setReminder(emptyReminderDraft(remindersMode));
        if (flash) {
          setSavedFlash(true);
          window.setTimeout(() => {
            setSavedFlash(false);
            void hide(false);
          }, 400);
        } else {
          await hide(false);
        }
        return true;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        hidingRef.current = false;
        return false;
      } finally {
        setSaving(false);
      }
    },
    [hide, remindersMode],
  );

  /** Esc / blur / hide: either auto-save or keep draft. */
  const dismiss = useCallback(async () => {
    if (saving || savedFlash) return;
    const value = textRef.current;
    const pid = selectedIdRef.current;
    if (saveOnHide && value.trim() && pid) {
      await save({ flash: false });
      return;
    }
    await hide(true);
  }, [hide, save, saveOnHide, saving, savedFlash]);

  const handleBlurHide = useCallback(async () => {
    if (!config?.settings.hideOnBlur) return;
    if (saving || savedFlash || openList) return;
    await dismiss();
  }, [config?.settings.hideOnBlur, dismiss, openList, saving, savedFlash]);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    void listen("quick-shown", async () => {
      hidingRef.current = false;
      setVisible(true);
      setSavedFlash(false);
      setError(null);
      setOpenList(false);
      await load();
      requestAnimationFrame(() => textareaRef.current?.focus());
    }).then((u) => unsubs.push(u));
    void listen("quick-hidden", () => setVisible(false)).then((u) =>
      unsubs.push(u),
    );
    void listen<AppConfig>("config-updated", (e) => {
      setConfig(e.payload);
      applyTheme(e.payload.settings.theme, e.payload.settings.accentColor);
    }).then((u) => unsubs.push(u));
    void listen("quick-blur", () => {
      void handleBlurHide();
    }).then((u) => unsubs.push(u));
    return () => unsubs.forEach((u) => u());
  }, [load, handleBlurHide]);

  // Detect initial visibility if window was shown before React mounted.
  useEffect(() => {
    void getCurrentWindow()
      .isVisible()
      .then((v) => {
        if (v) {
          setVisible(true);
          requestAnimationFrame(() => textareaRef.current?.focus());
        }
      });
  }, []);

  const selectProject = async (p: Project) => {
    setProjectId(p.id);
    setOpenList(false);
    setProjectQuery("");
    await api.setLastProject(p.id);
    scheduleDraft(text, p.id);
    textareaRef.current?.focus();
  };

  const cycleProject = (dir: 1 | -1) => {
    if (!projects.length) return;
    const idx = Math.max(
      0,
      projects.findIndex((p) => p.id === (selected?.id ?? "")),
    );
    const next = projects[(idx + dir + projects.length) % projects.length];
    void selectProject(next);
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void dismiss();
        return;
      }
      if (matchesHotkey(e, saveHotkey)) {
        e.preventDefault();
        void save({ flash: true });
        return;
      }
      if (e.ctrlKey && (e.key === "p" || e.key === "P")) {
        e.preventDefault();
        setOpenList(true);
        return;
      }
      if (e.ctrlKey && e.key === "ArrowUp") {
        e.preventDefault();
        cycleProject(-1);
        return;
      }
      if (e.ctrlKey && e.key === "ArrowDown") {
        e.preventDefault();
        cycleProject(1);
        return;
      }
      if (e.ctrlKey && /^[1-9]$/.test(e.key)) {
        e.preventDefault();
        const p = projects[Number(e.key) - 1];
        if (p) void selectProject(p);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [text, selected, projects, config, saveHotkey, dismiss, save]);

  const glass = config?.settings.glassEffect ?? true;

  return (
    <div className="h-full w-full p-3 select-none" data-tauri-drag-region>
      <AnimatePresence>
        {visible && (
          <motion.div
            initial={{ opacity: 0, scale: 0.96 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.96 }}
            transition={{ duration: 0.16 }}
            className={`relative flex h-full flex-col overflow-hidden rounded-2xl border border-line/70 shadow-glass ${
              glass ? "glass-fallback" : "bg-surface-elevated"
            }`}
            onMouseDown={(e) => e.stopPropagation()}
          >
            <div className="flex items-center gap-2 border-b border-line/60 px-3 py-2">
              <button
                type="button"
                className="flex min-w-0 flex-1 items-center gap-2 rounded-lg px-2 py-1.5 text-left hover:bg-surface-muted/70"
                onClick={() => setOpenList((v) => !v)}
              >
                <span
                  className="h-2.5 w-2.5 shrink-0 rounded-full"
                  style={{ background: selected?.color ?? "#888" }}
                />
                <span className="text-base">{selected?.emoji ?? "📝"}</span>
                <span className="truncate text-sm font-medium text-ink">
                  {selected?.name ?? "Нет проектов"}
                </span>
                <ChevronDown className="ml-auto h-4 w-4 text-ink-faint" />
              </button>
              <span className="text-xs text-ink-faint tabular-nums">
                {text.length}
              </span>
            </div>

            <div className="relative min-h-0 flex-1">
              <textarea
                ref={textareaRef}
                value={text}
                onChange={(e) => {
                  setText(e.target.value);
                  setError(null);
                  scheduleDraft(e.target.value, selected?.id ?? null);
                }}
                placeholder="Быстрая заметка…"
                className="h-full w-full resize-none bg-transparent px-4 py-3 text-[15px] leading-relaxed text-ink outline-none placeholder:text-ink-faint"
              />

              <AnimatePresence>
                {openList && (
                  <motion.div
                    initial={{ opacity: 0, y: -4 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -4 }}
                    className="absolute inset-x-2 top-2 z-20 max-h-44 overflow-auto rounded-xl border border-line bg-surface-elevated p-2 shadow-lg"
                  >
                    <div className="mb-2 flex items-center gap-2 rounded-lg bg-surface-muted px-2 py-1.5">
                      <Search className="h-3.5 w-3.5 text-ink-faint" />
                      <input
                        autoFocus
                        value={projectQuery}
                        onChange={(e) => setProjectQuery(e.target.value)}
                        placeholder="Поиск проекта…"
                        className="w-full bg-transparent text-sm outline-none"
                      />
                    </div>
                    {projects.map((p, i) => (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => void selectProject(p)}
                        className={`flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-sm hover:bg-surface-muted ${
                          selected?.id === p.id ? "bg-surface-muted" : ""
                        }`}
                      >
                        <span
                          className="h-2.5 w-2.5 rounded-full"
                          style={{ background: p.color }}
                        />
                        <span>{p.emoji}</span>
                        <span className="flex-1 truncate">{p.name}</span>
                        {i < 9 && (
                          <span className="text-[10px] text-ink-faint">
                            Ctrl+{i + 1}
                          </span>
                        )}
                      </button>
                    ))}
                    {!projects.length && (
                      <div className="px-2 py-3 text-center text-xs text-ink-muted">
                        Проектов нет — откройте главное окно
                      </div>
                    )}
                  </motion.div>
                )}
              </AnimatePresence>

              <AnimatePresence>
                {savedFlash && (
                  <motion.div
                    initial={{ opacity: 0, scale: 0.9 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0 }}
                    className="absolute inset-0 flex items-center justify-center bg-surface-elevated/70"
                  >
                    <div className="flex items-center gap-2 rounded-full bg-emerald-500/15 px-4 py-2 text-emerald-400">
                      <Check className="h-5 w-5" />
                      <span className="text-sm font-medium">Сохранено</span>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            {remindersMode !== "off" && (
              <div
                className="border-t border-line/60"
                onMouseDown={(e) => e.stopPropagation()}
              >
                <ReminderEditor
                  mode={remindersMode}
                  value={reminder}
                  onChange={setReminder}
                  compact
                />
              </div>
            )}

            <div className="flex items-center justify-between border-t border-line/60 px-3 py-2 text-[11px] text-ink-faint">
              <div className="flex gap-3">
                <span>
                  <kbd className="rounded bg-surface-muted px-1">{saveHotkey}</kbd>{" "}
                  сохранить
                </span>
                <span>
                  <kbd className="rounded bg-surface-muted px-1">Esc</kbd>{" "}
                  {saveOnHide ? "сохранить и закрыть" : "закрыть"}
                </span>
              </div>
              {error && <span className="text-rose-400">{error}</span>}
              {saving && <span>Сохранение…</span>}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
