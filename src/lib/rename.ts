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

/** "209 bestanden in 4 maanden (75,0 GB), 86 sidecars, 183 previews". Which backup there is, is
 * none of the app's business: the tab says in general that one by path copies them again. */
export function renameSummary(p: RenamePlan): string {
  if (p.files === 0) return "niets te hernoemen";
  const parts = [
    `${count(p.files)} bestanden in ${count(p.months.length)} ${p.months.length === 1 ? "maand" : "maanden"} (${gb(p.bytes)} GB)`,
    p.sidecars ? `${count(p.sidecars)} sidecars` : null,
    p.previews ? `${count(p.previews)} previews` : null,
    p.duplicates ? `${count(p.duplicates)} dubbel, blijven staan` : null,
    p.conflicts ? `${count(p.conflicts)} naamconflicten` : null,
  ];
  return parts.filter(Boolean).join(", ");
}

/** Ticks or unticks a source. */
export function toggleSource(chosen: string[], s: string): string[] {
  return chosen.includes(s) ? chosen.filter((x) => x !== s) : [...chosen, s];
}

/** How far, as "42%", from 0 to 1. */
export const percent = (f: number): string => `${Math.floor(Math.min(1, Math.max(0, f)) * 100)}%`;
