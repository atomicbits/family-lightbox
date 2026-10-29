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

/** How many photos and videos a month shows. */
export const shownCount = (m: MonthSummary): number => m.photos + m.videos - (m.hidden ?? 0);

/** A month's items without the hidden ones. */
export function visibleItems(items: Item[], hidden: Record<string, unknown>): Item[] {
  return items.some((i) => i.hash in hidden) ? items.filter((i) => !(i.hash in hidden)) : items;
}

/** "312 foto's en video's, 20 verborgen". */
export function countTitle(shown: number, hidden: number): string {
  const base = `${shown} ${shown === 1 ? "foto of video" : "foto's en video's"}`;
  return hidden > 0 ? `${base}, ${hidden} verborgen` : base;
}
