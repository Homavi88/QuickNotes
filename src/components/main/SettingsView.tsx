import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../store/useAppStore";
import type { Settings, Theme } from "../../types";
import { PROJECT_COLORS } from "../../types";

export function SettingsView() {
  const { config, updateSettings } = useAppStore();
  const [local, setLocal] = useState<Settings | null>(null);
  const [recording, setRecording] = useState(false);
  const [hotkeyError, setHotkeyError] = useState<string | null>(null);
  const [savedMsg, setSavedMsg] = useState<string | null>(null);

  useEffect(() => {
    if (config) setLocal({ ...config.settings });
  }, [config]);

  if (!local) return null;

  const patch = (partial: Partial<Settings>) =>
    setLocal((s) => (s ? { ...s, ...partial } : s));

  const save = async () => {
    setHotkeyError(null);
    setSavedMsg(null);
    try {
      await invoke("reregister_hotkey", { hotkey: local.hotkey });
      if (local.autostart) await enableAutostart();
      else await disableAutostart();
      await updateSettings(local);
      setSavedMsg("Сохранено");
      window.setTimeout(() => setSavedMsg(null), 1500);
    } catch (e) {
      setHotkeyError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-auto p-6">
      <div className="mb-6">
        <h2 className="text-xl font-semibold">Настройки</h2>
        <p className="text-sm text-ink-muted">
          Хоткей, автозапуск, тема и хранение заметок.
        </p>
      </div>

      <div className="mx-auto w-full max-w-2xl space-y-5">
        <Field label="Глобальный хоткей">
          <button
            type="button"
            onClick={() => setRecording(true)}
            className="rounded-xl border border-line bg-surface-muted px-3 py-2 text-sm"
          >
            {recording ? "Нажмите сочетание…" : local.hotkey}
          </button>
          {recording && (
            <HotkeyRecorder
              onCancel={() => setRecording(false)}
              onRecord={(hotkey) => {
                patch({ hotkey });
                setRecording(false);
              }}
            />
          )}
          {hotkeyError && (
            <div className="mt-1 text-xs text-rose-400">{hotkeyError}</div>
          )}
        </Field>

        <Toggle
          label="Автозапуск с Windows"
          checked={local.autostart}
          onChange={(autostart) => patch({ autostart })}
        />

        <Field label="Тема">
          <div className="flex gap-2">
            {(
              [
                ["dark", "Тёмная"],
                ["light", "Светлая"],
                ["system", "Системная"],
              ] as [Theme, string][]
            ).map(([value, label]) => (
              <button
                key={value}
                type="button"
                onClick={() => patch({ theme: value })}
                className={`rounded-xl px-3 py-2 text-sm ${
                  local.theme === value
                    ? "bg-accent text-white"
                    : "border border-line bg-surface-muted"
                }`}
              >
                {label}
              </button>
            ))}
          </div>
        </Field>

        <Field label="Акцентный цвет">
          <div className="flex flex-wrap gap-2">
            {PROJECT_COLORS.map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => patch({ accentColor: c })}
                className={`h-7 w-7 rounded-full border-2 ${
                  local.accentColor === c ? "border-ink" : "border-transparent"
                }`}
                style={{ background: c }}
              />
            ))}
            <input
              type="color"
              value={local.accentColor}
              onChange={(e) => patch({ accentColor: e.target.value })}
              className="h-7 w-10 cursor-pointer rounded border border-line bg-transparent"
            />
          </div>
        </Field>

        <Field label="Папка хранения заметок">
          <div className="flex flex-wrap gap-2">
            <input
              readOnly
              value={local.storageDir}
              className="min-w-0 flex-1 rounded-xl border border-line bg-surface-muted px-3 py-2 text-sm"
            />
            <button
              type="button"
              className="rounded-xl border border-line px-3 py-2 text-sm hover:bg-surface-muted"
              onClick={async () => {
                const dir = await open({
                  directory: true,
                  multiple: false,
                  defaultPath: local.storageDir,
                });
                if (typeof dir === "string") patch({ storageDir: dir });
              }}
            >
              Выбрать…
            </button>
            <button
              type="button"
              className="rounded-xl border border-line px-3 py-2 text-sm hover:bg-surface-muted"
              onClick={() => void openPath(local.storageDir)}
            >
              Открыть
            </button>
          </div>
        </Field>

        <Toggle
          label="Скрывать быстрый ввод при потере фокуса"
          checked={local.hideOnBlur}
          onChange={(hideOnBlur) => patch({ hideOnBlur })}
        />

        <Toggle
          label="Эффект стекла (acrylic/mica)"
          checked={local.glassEffect}
          onChange={(glassEffect) => patch({ glassEffect })}
        />

        <div className="flex items-center gap-3 pt-2">
          <button
            type="button"
            onClick={() => void save()}
            className="rounded-xl bg-accent px-4 py-2 text-sm font-medium text-white"
          >
            Сохранить настройки
          </button>
          {savedMsg && <span className="text-sm text-emerald-400">{savedMsg}</span>}
          <AutostartHint />
        </div>
      </div>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-2xl border border-line bg-surface-elevated p-4">
      <div className="mb-2 text-sm font-medium">{label}</div>
      {children}
    </div>
  );
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between gap-4 rounded-2xl border border-line bg-surface-elevated p-4 text-sm">
      <span>{label}</span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="h-4 w-4"
      />
    </label>
  );
}

function HotkeyRecorder({
  onRecord,
  onCancel,
}: {
  onRecord: (hotkey: string) => void;
  onCancel: () => void;
}) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        onCancel();
        return;
      }
      const parts: string[] = [];
      if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      const key = normalizeKey(e);
      if (!key) return;
      if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return;
      parts.push(key);
      onRecord(parts.join("+"));
    };
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [onRecord, onCancel]);

  return (
    <div className="mt-2 text-xs text-ink-muted">Ожидание… Esc — отмена</div>
  );
}

function normalizeKey(e: KeyboardEvent): string | null {
  if (e.code.startsWith("Key") && e.code.length === 4) return e.code.slice(3);
  if (e.code.startsWith("Digit")) return e.code.slice(5);
  const map: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Escape: "Esc",
  };
  if (map[e.code]) return map[e.code];
  if (e.key.length === 1) return e.key.toUpperCase();
  return null;
}

function AutostartHint() {
  const [enabled, setEnabled] = useState<boolean | null>(null);
  useEffect(() => {
    void isAutostartEnabled()
      .then(setEnabled)
      .catch(() => setEnabled(null));
  }, []);
  if (enabled === null) return null;
  return (
    <span className="text-xs text-ink-faint">
      Автозапуск сейчас: {enabled ? "вкл" : "выкл"}
    </span>
  );
}
