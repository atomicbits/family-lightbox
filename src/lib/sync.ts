// The background sync in the UI (F007/S05): state from the Rust runner's events, and the texts for
// the line in the header and the confirmation dialog.
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { count as i18nCount, decimal, t } from "../i18n";

export type SyncPhase =
  | "idle" | "manifests" | "lists" | "counting" | "awaiting_confirmation" | "copying" | "done" | "cancelled" | "failed";

export type PreviewsSummary = {
  copy_files: number;
  copy_bytes: number;
  kept: number;
  skipped: string[];
  free: number | null;
  problem: string | null;
  needs_confirmation: boolean;
};

export type SyncState = {
  phase: SyncPhase;
  running: boolean;
  folder: string;
  files_done: number;
  files_total: number;
  bytes_done: number;
  bytes_total: number;
  summary: PreviewsSummary | null;
  manifests_copied: number;
  lists_copied: number;
  manifests_changed: string[];
  previews_copied: number;
  previews_bytes: number;
  /** Files skipped after failing twice (F007/S06), and some of them with their error. */
  failed_count: number;
  failed: string[];
  skipped: string[];
  message: string | null;
};

/** Measured in F007/S02 and S04: copying previews over SMB runs at about 6 MB/s. */
const BYTES_PER_SECOND = 6_000_000;

/** 13076 → "13 076". */
export function count(n: number): string {
  return i18nCount(n);
}

/** 4_990_000_000 → "5.0" / "5,0". */
export function gb(bytes: number): string {
  return decimal(bytes / 1e9);
}

export function estimate(bytes: number): string {
  const minutes = Math.round(bytes / BYTES_PER_SECOND / 60);
  if (minutes < 1) return t().settings.sync.lessThanMinute;
  if (minutes === 1) return t().settings.sync.aboutMinute;
  return t().settings.sync.aboutMinutes(minutes);
}

export function confirmText(s: PreviewsSummary): string {
  return t().settings.sync.confirm(gb(s.copy_bytes), s.copy_files, estimate(s.copy_bytes));
}

export type SyncLine = {
  text: string;
  tone: "muted" | "amber";
  cancellable: boolean;
  /** Clicking the text opens the confirmation dialog. */
  confirm: boolean;
};

/** The line next to the NAS indicator, or null when there is nothing to say. */
export function syncLine(s: SyncState | null): SyncLine | null {
  if (!s) return null;
  const base = { tone: "muted" as const, cancellable: false, confirm: false };
  const x = t().settings.sync;
  switch (s.phase) {
    case "manifests":
      return { ...base, text: s.files_total > 0 ? x.manifestsOf(s.files_done, s.files_total) : x.manifests(s.files_done), cancellable: true };
    case "lists":
      return { ...base, text: x.lists, cancellable: true };
    case "counting":
      return { ...base, text: x.counting(s.folder.split("/").slice(0, 2).join("/")).trimEnd(), cancellable: true };
    case "copying":
      return {
        ...base,
        text: x.copying(s.files_done, s.files_total, gb(s.bytes_done), gb(s.bytes_total)),
        cancellable: true,
      };
    case "awaiting_confirmation":
      // a large first pass on this Mac waits for a click (F007/S04, over 5 GB since 2026-11-03)
      return { ...base, text: s.summary ? x.ready(gb(s.summary.copy_bytes)) : x.readyNoSize, confirm: true };
    case "failed":
      return { ...base, tone: "amber", text: s.message ?? x.failed };
    case "cancelled":
      return { ...base, text: x.cancelled };
    case "done": {
      if (s.manifests_copied === 0 && s.lists_copied === 0 && s.previews_copied === 0 && s.failed_count === 0) return null;
      const skipped = s.failed_count > 0 ? ` · ${x.skipped(s.failed_count)}` : "";
      const parts = [
        s.manifests_copied > 0 && x.manifestsCopied(s.manifests_copied),
        s.lists_copied > 0 && x.listsCopied,
        s.previews_copied > 0 && x.previewsCopied(s.previews_copied),
      ].filter(Boolean);
      return { ...base, text: parts.length ? `${x.updated(parts.join(", "))}${skipped}` : x.skipped(s.failed_count) };
    }
    default:
      return null;
  }
}

/** Whether the line goes after a few seconds: done, cancelled, and a failure, except too little
 * disk space, which asks for an action like the confirmation does (F016/S02). */
export function fades(s: SyncState): boolean {
  if (s.phase === "done" || s.phase === "cancelled") return true;
  return s.phase === "failed" && !/tekort|short/.test(s.message ?? "");
}

export type Sync = {
  state: SyncState | null;
  /** The line to show (done and cancelled fade after a few seconds). */
  line: SyncLine | null;
  /** `atLaunch`: the sync at start, which runs once per app process, not again after a reload. */
  start: (kind: "all" | "previews", confirmed: boolean, atLaunch?: boolean) => Promise<void>;
  cancel: () => Promise<void>;
};

export function useSync(): Sync {
  const [state, setState] = useState<SyncState | null>(null);
  const [visible, setVisible] = useState(true);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    invoke<SyncState>("sync_status").then(setState).catch(() => {});
    const show = (s: SyncState) => {
      setState(s);
      setVisible(true);
      if (timer.current) clearTimeout(timer.current);
      // news for a moment, the details in the messages (F016/S02); what asks for an action stays
      if (fades(s)) timer.current = setTimeout(() => setVisible(false), 8000);
    };
    const off = [listen<SyncState>("sync-progress", (e) => show(e.payload)), listen<SyncState>("sync-done", (e) => show(e.payload))];
    return () => { off.forEach((p) => p.then((f) => f())); if (timer.current) clearTimeout(timer.current); };
  }, []);

  const start = useCallback(async (kind: "all" | "previews", confirmed: boolean, atLaunch = false) => {
    await invoke("sync_start", { kind, confirmed, atLaunch });
  }, []);
  const cancel = useCallback(async () => { await invoke("sync_cancel"); }, []);

  return { state, line: visible ? syncLine(state) : null, start, cancel };
}
