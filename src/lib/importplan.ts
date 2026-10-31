// The import view's texts (F018/S04): what a plan of one source says.
import { count, gb } from "./sync";

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
};
export type PlanProgress = { files_done: number; files_total: number; bytes_done: number; bytes_total: number };

/** "82 nieuw (93,0 GB) · 71 al in de bibliotheek · 1 naamconflict" or "niets nieuw". */
export function planSummary(p: SourcePlan): string {
  const parts = [
    p.copy_files > 0 ? `${count(p.copy_files)} nieuw (${gb(p.copy_bytes)} GB)` : "niets nieuw",
    p.duplicates > 0 ? `${count(p.duplicates)} al in de bibliotheek` : null,
    p.conflicts > 0 ? `${count(p.conflicts)} ${p.conflicts === 1 ? "naamconflict" : "naamconflicten"}` : null,
  ];
  return parts.filter(Boolean).join(" · ");
}

/** "2025/03 (2) · 2026/08 (59)". */
export function monthsText(m: Record<string, number>): string {
  return Object.entries(m).map(([k, n]) => `${k} (${count(n)})`).join(" · ");
}

const WHY: Record<string, string> = { partner: "van het bijhorende bestand", name: "uit de bestandsnaam", mtime: "uit de wijzigingstijd" };

/** "datum niet uit het bestand: 3 uit de wijzigingstijd, 1 uit de bestandsnaam", or "". */
export function datesText(m: Record<string, number>): string {
  const parts = Object.entries(m).map(([k, n]) => `${count(n)} ${WHY[k] ?? k}`);
  return parts.length ? `datum niet uit het bestand: ${parts.join(", ")}` : "";
}

/** What happens to one file, short. */
export function actionText(a: Action): string {
  if (a.what === "new") return "nieuw";
  if (a.what === "duplicate") return `al in de bibliotheek: ${a.of}`;
  return `naam bestaat al (${a.existing}): hernoemd`;
}

/** "1 234 / 5 000 bestanden · 1,2 / 93,0 GB". */
export function progressText(p: PlanProgress): string {
  return `${count(p.files_done)} / ${count(p.files_total)} bestanden · ${gb(p.bytes_done)} / ${gb(p.bytes_total)} GB`;
}
