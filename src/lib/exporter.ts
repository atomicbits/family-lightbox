// Exporting a list (F024/S03): the presets, the plan's text, and the job kept by the app, so the
// window may be closed and the app restarted; an interrupted export goes on by itself.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { count } from "./sync";

export type Preset = { id: string; label: string; what: string; long: number | null; quality: number; dpi: number | null; originals: boolean };
export type ExportPlan = { photos: number; videos: number; files: number; dest: string | null };
export type ExportProgress = { done: number; total: number };
export type ExportDone = { files: number; dest: string; problems: string[]; error: string | null };

/** "Fotoboek A4 · 3 508 px" for a preset's choice. */
export function presetText(p: Preset): string {
  if (p.originals) return `${p.label} · ongewijzigd`;
  return `${p.label} · ${p.long ? `${count(p.long)} px` : "volle grootte"}`;
}

/** "602 foto's als JPEG, 33 video's ongewijzigd". */
export function exportPlanText(p: ExportPlan, preset: Preset): string {
  const photos = `${count(p.photos)} ${p.photos === 1 ? "foto" : "foto's"}`;
  const videos = p.videos ? `, ${count(p.videos)} ${p.videos === 1 ? "video" : "video's"} ongewijzigd` : "";
  return preset.originals ? `${count(p.files)} bestanden: de originelen met hun sidecars` : `${photos} als JPEG${videos}`;
}

/** The last two parts of a folder, for the dialog: "2026/2026-11-aosta-fotoboek-a4". */
export function shortFolder(path: string): string {
  return path.split("/").filter(Boolean).slice(-2).join("/");
}

export function exportDoneText(d: ExportDone): string {
  if (d.error) return `exporteren gestopt: ${d.error}`;
  const n = `${count(d.files)} bestanden geëxporteerd naar ${shortFolder(d.dest)}`;
  return d.problems.length ? `${n} · ${count(d.problems.length)} niet (zie de meldingen)` : n;
}

export type Exporter = {
  running: ExportProgress | null;
  start: (list: string, preset: string, dest: string | null) => Promise<void>;
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
  const start = useCallback(async (list: string, preset: string, dest: string | null) => {
    await invoke("export_start", { list, preset, dest });
    setRunning({ done: 0, total: 0 });
  }, []);
  const resume = useCallback(async () => {
    if (await invoke<boolean>("export_pending")) await start("", "", null);
  }, [start]);
  return { running, start, resume };
}
