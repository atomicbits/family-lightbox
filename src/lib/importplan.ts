// The import view's texts (F018/S04): what a plan of one source says.
import { count, gb } from "./sync";
import { t } from "../i18n";

export type Action = { what: "new" } | { what: "duplicate"; of: string } | { what: "conflict"; existing: string };
export type Planned = { src: string; target: string; size: number; hash: string; kind: string; taken: string; taken_from: string; action: Action };
export type SourcePlan = {
  name: string;
  path: string;
  files: Planned[];
  copy_files: number;
  copy_bytes: number;
  duplicates: number;
  conflicts: number;
  excluded: number;
  months: Record<string, number>;
  not_from_file: Record<string, number>;
  problems: string[];
  /** A synced folder (S06): copied only; what was taken is recorded. */
  synced?: boolean;
  taken_before?: number;
  older?: number;
  from_month?: string | null;
  read?: number;
  live_skipped?: number;
};
export type PlanProgress = { files_done: number; files_total: number; bytes_done: number; bytes_total: number };

/** "82 nieuw (93,0 GB) · 71 al in de bibliotheek · 1 naamconflict" or "niets nieuw". */
export function planSummary(p: SourcePlan): string {
  const tx = t().imports.plan;
  const parts = [
    p.copy_files > 0 ? tx.newFiles(p.copy_files, gb(p.copy_bytes)) : tx.nothingNew,
    p.duplicates > 0 ? tx.inLibrary(p.duplicates) : null,
    p.conflicts > 0 ? tx.conflicts(p.conflicts) : null,
  ];
  return parts.filter(Boolean).join(" · ");
}

/** "2025/03 (2) · 2026/08 (59)". */
export function monthsText(m: Record<string, number>): string {
  return Object.entries(m).map(([k, n]) => `${k} (${count(n)})`).join(" · ");
}

/** "datum niet uit het bestand: 3 uit de wijzigingstijd, 1 uit de bestandsnaam", or "". */
export function datesText(m: Record<string, number>): string {
  const tx = t().imports.plan;
  const parts = Object.entries(m).map(([k, n]) => `${count(n)} ${tx.why[k] ?? k}`);
  return parts.length ? tx.datesNotFromFile(parts.join(", ")) : "";
}

/** What happens to one file, short. */
export function actionText(a: Action): string {
  const tx = t().imports.plan;
  if (a.what === "new") return tx.actionNew;
  if (a.what === "duplicate") return tx.actionDuplicate(a.of);
  return tx.actionConflict(a.existing);
}

/** "1 234 / 5 000 bestanden · 1,2 / 93,0 GB". */
export function progressText(p: PlanProgress): string {
  return t().imports.plan.progress(p.files_done, p.files_total, gb(p.bytes_done), gb(p.bytes_total));
}

/** For a synced folder (S06): "1 234 al eerder genomen · 3 456 ouder dan 2019/11 · 12 gelezen", or "". */
export function syncedText(p: SourcePlan): string {
  if (!p.synced) return "";
  const tx = t().imports.plan;
  const parts = [
    p.taken_before ? tx.takenBefore(p.taken_before) : null,
    p.older && p.from_month ? tx.older(p.older, p.from_month) : null,
    p.live_skipped ? tx.liveSkipped(p.live_skipped) : null,
    tx.read(p.read ?? 0),
  ];
  return parts.filter(Boolean).join(" · ");
}
