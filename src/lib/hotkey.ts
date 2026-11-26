/** Parse "Ctrl+Enter" / "Enter" style combos used in Settings + Quick window. */

const MODIFIERS = new Set(["ctrl", "control", "cmd", "meta", "alt", "shift"]);

export interface ParsedHotkey {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  key: string;
}

export function parseHotkey(hotkey: string): ParsedHotkey | null {
  const parts = hotkey
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean);
  if (!parts.length) return null;

  let ctrl = false;
  let alt = false;
  let shift = false;
  const keyParts: string[] = [];

  for (const part of parts) {
    const lower = part.toLowerCase();
    if (lower === "ctrl" || lower === "control" || lower === "cmd" || lower === "meta") {
      ctrl = true;
    } else if (lower === "alt") {
      alt = true;
    } else if (lower === "shift") {
      shift = true;
    } else {
      keyParts.push(part);
    }
  }

  if (keyParts.length !== 1) return null;
  return { ctrl, alt, shift, key: keyParts[0] };
}

/** Normalize a KeyboardEvent to the same key token used in hotkey strings. */
export function eventKeyToken(e: KeyboardEvent): string | null {
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

export function matchesHotkey(e: KeyboardEvent, hotkey: string): boolean {
  const parsed = parseHotkey(hotkey);
  if (!parsed) return false;
  const token = eventKeyToken(e);
  if (!token) return false;
  if (token.toLowerCase() !== parsed.key.toLowerCase()) return false;
  // Treat Meta as Ctrl for Windows-oriented combos recorded as "Ctrl+…"
  const ctrlDown = e.ctrlKey || e.metaKey;
  if (ctrlDown !== parsed.ctrl) return false;
  if (e.altKey !== parsed.alt) return false;
  if (e.shiftKey !== parsed.shift) return false;
  return true;
}

export function formatHotkeyLabel(hotkey: string): string {
  return hotkey.trim() || "Ctrl+Enter";
}

export function isModifierOnlyKey(e: KeyboardEvent): boolean {
  return ["Control", "Alt", "Shift", "Meta"].includes(e.key);
}

export { MODIFIERS };
