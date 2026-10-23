import type { MonthSummary } from "./api";

const MAANDEN = ["januari", "februari", "maart", "april", "mei", "juni", "juli", "augustus",
  "september", "oktober", "november", "december"];

/** "2026/07" → "juli"; een mapnaam zonder maand ("2012/scans Peter") → "scans Peter". */
export function monthLabel(month: string): string {
  const m = /^(\d{4})\/(\d{2})$/.exec(month);
  if (m) return MAANDEN[Number(m[2]) - 1] ?? month;
  return month.replace(/^\d{4}\//, "");
}

export function yearOf(month: string): string {
  return month.slice(0, 4);
}

export type YearGroup = { year: string; months: MonthSummary[]; photos: number; videos: number };

/** Groepeert per jaar, nieuwste jaar eerst; de volgorde binnen een jaar is die van de invoer. */
export function groupByYear(months: MonthSummary[]): YearGroup[] {
  const groups = new Map<string, YearGroup>();
  for (const m of months) {
    const y = yearOf(m.month);
    const g = groups.get(y) ?? { year: y, months: [], photos: 0, videos: 0 };
    g.months.push(m);
    g.photos += m.photos;
    g.videos += m.videos;
    groups.set(y, g);
  }
  return [...groups.values()].sort((a, b) => b.year.localeCompare(a.year));
}

/** "2026-07-30T10:05:00+02:00" → "30 juli 2026, 10:05" (lokale tijd van de opname, zone genegeerd). */
export function takenLabel(taken: string | null): string {
  if (!taken) return "";
  const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})/.exec(taken);
  if (!m) return taken;
  return `${Number(m[3])} ${MAANDEN[Number(m[2]) - 1]} ${m[1]}, ${m[4]}:${m[5]}`;
}

export function fileName(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}
