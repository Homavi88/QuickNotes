import type { Note } from "../types";

function startOfDay(d: Date) {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

export function parseNoteDate(timestamp: string): Date {
  // "YYYY-MM-DD HH:mm"
  const [datePart, timePart = "00:00"] = timestamp.split(" ");
  const [y, m, d] = datePart.split("-").map(Number);
  const [hh, mm] = timePart.split(":").map(Number);
  return new Date(y, (m || 1) - 1, d || 1, hh || 0, mm || 0);
}

export function dayLabel(timestamp: string, now = new Date()): string {
  const date = parseNoteDate(timestamp);
  const diffDays = Math.round(
    (startOfDay(now) - startOfDay(date)) / (24 * 60 * 60 * 1000),
  );
  if (diffDays === 0) return "Сегодня";
  if (diffDays === 1) return "Вчера";
  return date.toLocaleDateString("ru-RU", {
    day: "numeric",
    month: "long",
    year: date.getFullYear() === now.getFullYear() ? undefined : "numeric",
  });
}

export function groupNotesByDay(notes: Note[]): { label: string; notes: Note[] }[] {
  const map = new Map<string, Note[]>();
  for (const note of notes) {
    const label = dayLabel(note.timestamp);
    const list = map.get(label) ?? [];
    list.push(note);
    map.set(label, list);
  }
  return Array.from(map.entries()).map(([label, grouped]) => ({
    label,
    notes: grouped,
  }));
}

export function highlightText(text: string, query: string): string {
  if (!query.trim()) return escapeHtml(text);
  const q = query.trim();
  const escaped = escapeHtml(text);
  const re = new RegExp(`(${escapeRegExp(escapeHtml(q))})`, "ig");
  return escaped.replace(re, '<mark class="mark-hit">$1</mark>');
}

function escapeHtml(s: string) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeRegExp(s: string) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
