// Renaming sources (F022/S05): the plan as the core sends it, and its texts.
import { gb } from "./sync";
import { t } from "../i18n";

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
  const r = t().settings.rename;
  if (p.files === 0) return r.nothing;
  const parts = [
    r.files(p.files, p.months.length, gb(p.bytes)),
    p.sidecars ? r.sidecars(p.sidecars) : null,
    p.previews ? r.previews(p.previews) : null,
    p.duplicates ? r.duplicates(p.duplicates) : null,
    p.conflicts ? r.conflicts(p.conflicts) : null,
  ];
  return parts.filter(Boolean).join(", ");
}

/** Ticks or unticks a source. */
export function toggleSource(chosen: string[], s: string): string[] {
  return chosen.includes(s) ? chosen.filter((x) => x !== s) : [...chosen, s];
}

/** How far, as "42%", from 0 to 1. */
export const percent = (f: number): string => `${Math.floor(Math.min(1, Math.max(0, f)) * 100)}%`;
