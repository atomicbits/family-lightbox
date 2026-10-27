// The background sync in the UI (F007/S05): state from the Rust runner's events, and the texts for
// the line in the header and the confirmation dialog.
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type SyncPhase =
  | "idle" | "manifests" | "counting" | "awaiting_confirmation" | "copying" | "done" | "cancelled" | "failed";

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
  manifests_changed: string[];
  previews_copied: number;
  previews_bytes: number;
  skipped: string[];
  message: string | null;
};

/** Measured in F007/S02 and S04: copying previews over SMB runs at about 6 MB/s. */
const BYTES_PER_SECOND = 6_000_000;

/** 13076 → "13 076". */
export function count(n: number): string {
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
}

/** 4_990_000_000 → "5,0". */
export function gb(bytes: number): string {
  return (bytes / 1e9).toFixed(1).replace(".", ",");
}

export function estimate(bytes: number): string {
  const minutes = Math.round(bytes / BYTES_PER_SECOND / 60);
  if (minutes < 1) return "minder dan een minuut";
  if (minutes === 1) return "ongeveer een minuut";
  return `ongeveer ${minutes} minuten`;
}

export function confirmText(s: PreviewsSummary): string {
  return `${gb(s.copy_bytes)} GB previews ophalen (${count(s.copy_files)} bestanden)? Dat duurt ${estimate(s.copy_bytes)}.`;
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
  switch (s.phase) {
    case "manifests":
      return { ...base, text: s.files_total > 0 ? `manifesten ${count(s.files_done)}/${count(s.files_total)}` : `manifesten ${count(s.files_done)}`, cancellable: true };
    case "counting":
      return { ...base, text: `previews tellen… ${s.folder.split("/").slice(0, 2).join("/")}`.trimEnd(), cancellable: true };
    case "copying":
      return {
        ...base,
        text: `previews ${count(s.files_done)}/${count(s.files_total)} · ${gb(s.bytes_done)}/${gb(s.bytes_total)} GB`,
        cancellable: true,
      };
    case "awaiting_confirmation":
      return { ...base, text: "Previews ophalen…", confirm: true };
    case "failed":
      return { ...base, tone: "amber", text: s.message ?? "synchroniseren mislukt" };
    case "cancelled":
      return { ...base, text: "geannuleerd" };
    case "done": {
      if (s.manifests_copied === 0 && s.previews_copied === 0) return null;
      const parts = [
        s.manifests_copied > 0 && `${count(s.manifests_copied)} ${s.manifests_copied === 1 ? "manifest" : "manifesten"}`,
        s.previews_copied > 0 && `${count(s.previews_copied)} previews`,
      ].filter(Boolean);
      return { ...base, text: `bijgewerkt: ${parts.join(", ")}` };
    }
    default:
      return null;
  }
}

export type Sync = {
  state: SyncState | null;
  /** The line to show (done and cancelled fade after a few seconds). */
  line: SyncLine | null;
  start: (kind: "all" | "previews", confirmed: boolean) => Promise<void>;
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
      // done and cancelled are news for a moment; failures and the confirmation stay
      if (s.phase === "done" || s.phase === "cancelled") timer.current = setTimeout(() => setVisible(false), 8000);
    };
    const off = [listen<SyncState>("sync-progress", (e) => show(e.payload)), listen<SyncState>("sync-done", (e) => show(e.payload))];
    return () => { off.forEach((p) => p.then((f) => f())); if (timer.current) clearTimeout(timer.current); };
  }, []);

  const start = useCallback(async (kind: "all" | "previews", confirmed: boolean) => {
    await invoke("sync_start", { kind, confirmed });
  }, []);
  const cancel = useCallback(async () => { await invoke("sync_cancel"); }, []);

  return { state, line: visible ? syncLine(state) : null, start, cancel };
}
