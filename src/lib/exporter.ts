// Exporting a list (F024/S03): the presets, the plan's text, and the job kept by the app, so the
// window may be closed and the app restarted; an interrupted export goes on by itself.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { count } from "./sync";
import { t } from "../i18n";

export type Preset = { id: string; label: string; what: string; long: number | null; quality: number; dpi: number | null; originals: boolean };
/** An own setting (F024/S03): the long side, the quality 1–100, the DPI or none. */
export type Custom = { long: number; quality: number; dpi: number | null };

/** Why an own setting can't be used, or null; the same bounds as the core. */
export function customProblem(c: Custom): string | null {
  if (!Number.isInteger(c.long) || c.long < 100 || c.long > 20000) return t().lists.export.longProblem;
  if (!Number.isInteger(c.quality) || c.quality < 1 || c.quality > 100) return t().lists.export.qualityProblem;
  if (c.dpi !== null && (!Number.isInteger(c.dpi) || c.dpi < 72 || c.dpi > 1200)) return t().lists.export.dpiProblem;
  return null;
}

/** "3 000 px, kwaliteit 90, 300 dpi". */
export function customText(c: Custom): string {
  return t().lists.export.custom(count(c.long), c.quality, c.dpi);
}

export type ExportPlan = { photos: number; videos: number; files: number; dest: string | null };
export type ExportProgress = { done: number; total: number };
export type ExportDone = { files: number; dest: string; problems: string[]; error: string | null };

/** "Fotoboek A4 · 3 508 px" for a preset's choice. */
export function presetText(p: Preset): string {
  if (p.originals) return t().lists.export.unchanged(p.label);
  return `${p.label} · ${p.long ? `${count(p.long)} px` : t().lists.export.fullSize}`;
}

/** "602 foto's als JPEG, 33 video's ongewijzigd". */
export function exportPlanText(p: ExportPlan, preset: Preset): string {
  const e = t().lists.export;
  return preset.originals ? e.originals(p.files) : e.asJpeg(e.photos(p.photos), p.videos ? e.videos(p.videos) : "");
}

/** The last two parts of a folder, for the dialog: "2026/2026-11-aosta-fotoboek-a4". */
export function shortFolder(path: string): string {
  return path.split("/").filter(Boolean).slice(-2).join("/");
}

export function exportDoneText(d: ExportDone): string {
  if (d.error) return t().lists.export.stopped(d.error);
  const n = t().lists.export.done(d.files, shortFolder(d.dest));
  return d.problems.length ? `${n} · ${t().lists.notDone(d.problems.length)}` : n;
}

export type Exporter = {
  running: ExportProgress | null;
  start: (list: string, preset: string, dest: string | null, custom?: Custom | null) => Promise<void>;
  resume: () => Promise<void>;
};

export function useExporter(onDone: (d: ExportDone) => void): Exporter {
  const [running, setRunning] = useState<ExportProgress | null>(null);
  useEffect(() => {
    const offs = [
      listen<ExportProgress>("export-progress", (e) => setRunning(e.payload)),
      listen<ExportDone>("export-done", (e) => { setRunning(null); onDone(e.payload); }),
    ];
    return () => { offs.forEach((o) => o.then((f) => f())); };
  }, [onDone]);
  const start = useCallback(async (list: string, preset: string, dest: string | null, custom: Custom | null = null) => {
    await invoke("export_start", { list, preset, dest, custom });
    setRunning({ done: 0, total: 0 });
  }, []);
  const resume = useCallback(async () => {
    if (await invoke<boolean>("export_pending")) await start("", "", null);
  }, [start]);
  return { running, start, resume };
}
