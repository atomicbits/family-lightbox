// Hidden photos (F014/S02): left out of the grid and the counts, except in a list, where they
// are shown dimmed.
import type { Item, MonthSummary } from "./api";

/** The months with how many of their photos are hidden. */
export function withHidden(months: MonthSummary[], perMonth: Record<string, number>): MonthSummary[] {
  return months.map((m) => {
    const hidden = perMonth[m.month] ?? 0;
    return hidden === (m.hidden ?? 0) ? m : { ...m, hidden };
  });
}

/** How many photos and videos a month shows: its hidden ones too while they are shown (S04). */
export const shownCount = (m: MonthSummary): number => m.photos + m.videos - (m.revealed ? 0 : m.hidden ?? 0);

/** The months whose hidden photos are shown for now (F014/S04), marked. */
export function withRevealed(months: MonthSummary[], revealed: Set<string>): MonthSummary[] {
  return months.map((m) => {
    const r = revealed.has(m.month);
    return r === !!m.revealed ? m : { ...m, revealed: r };
  });
}

/** The set with these months shown (on) or hidden again. */
export function toggled(revealed: Set<string>, months: string[], on: boolean): Set<string> {
  const next = new Set(revealed);
  for (const m of months) {
    if (on) next.add(m);
    else next.delete(m);
  }
  return next;
}

/** A month's items without the hidden ones. */
export function visibleItems(items: Item[], hidden: Record<string, unknown>): Item[] {
  return items.some((i) => i.hash in hidden) ? items.filter((i) => !(i.hash in hidden)) : items;
}

/** "312 foto's en video's, 20 verborgen". */
export function countTitle(shown: number, hidden: number): string {
  const base = `${shown} ${shown === 1 ? "foto of video" : "foto's en video's"}`;
  return hidden > 0 ? `${base}, ${hidden} verborgen` : base;
}
