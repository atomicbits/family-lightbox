// Renaming sources (F022/S05): the plan as the core sends it, and its texts.
import { count, gb } from "./sync";

export type Move = { from: string; to: string; size: number; sidecar: boolean; preview: boolean };
export type MonthPlan = { month: string; moves: Move[]; duplicates: [string, string][]; conflicts: number };
export type RenamePlan = {
  from: string[];
  to: string;
  months: MonthPlan[];
  files: number;
  sidecars: number;
  previews: number;
  bytes: number;
  drive_bytes: number;
  duplicates: number;
  conflicts: number;
};

/** "209 bestanden in 4 maanden (75,0 GB), 86 sidecars, 183 previews · Drive laadt 74,3 GB opnieuw op". */
export function renameSummary(p: RenamePlan): string {
  if (p.files === 0) return "niets te hernoemen";
  const parts = [
    `${count(p.files)} bestanden in ${count(p.months.length)} ${p.months.length === 1 ? "maand" : "maanden"} (${gb(p.bytes)} GB)`,
    p.sidecars ? `${count(p.sidecars)} sidecars` : null,
    p.previews ? `${count(p.previews)} previews` : null,
    p.duplicates ? `${count(p.duplicates)} dubbel, blijven staan` : null,
    p.conflicts ? `${count(p.conflicts)} naamconflicten` : null,
  ];
  return `${parts.filter(Boolean).join(", ")} · Drive laadt ${gb(p.drive_bytes)} GB opnieuw op`;
}

/** Ticks or unticks a source. */
export function toggleSource(chosen: string[], s: string): string[] {
  return chosen.includes(s) ? chosen.filter((x) => x !== s) : [...chosen, s];
}
