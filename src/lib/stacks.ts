// Stacks in the grid (F015/S02): a stack's photos stand together at the place of its first photo,
// folded to the ones that matter or unfolded; which stacks are unfolded is kept in this window only.
import type { Item } from "./api";
import type { MarksState } from "./marks";

/** A photo of a stack, with the month it is in (which may differ from where its stack stands). */
export type Member = Item & { month: string };

export type Stack = {
  id: string;
  /** By date taken, then path. */
  members: Member[];
};

const byTaken = (a: Item, b: Item) => (a.taken ?? "").localeCompare(b.taken ?? "") || a.path.localeCompare(b.path);

/** The stacks from their members (from the index) and the marks' stack ids. */
export function buildStacks(members: Member[], ids: Record<string, string>): Map<string, Stack> {
  const out = new Map<string, Stack>();
  for (const m of members) {
    const id = ids[m.hash];
    if (!id) continue;
    if (!out.has(id)) out.set(id, { id, members: [] });
    out.get(id)!.members.push(m);
  }
  for (const s of out.values()) s.members.sort(byTaken);
  return out;
}

type Marks = Pick<MarksState, "ratings" | "hearts">;

/** What a folded stack shows: the photos with a heart or 4–5 stars; else those with the most
 * stars; else the first one. */
export function folded(members: Member[], marks: Marks): Member[] {
  if (members.length === 0) return [];
  const stars = (m: Member) => marks.ratings[m.hash]?.stars ?? 0;
  const top = members.filter((m) => Object.keys(marks.hearts[m.hash] ?? {}).length > 0 || stars(m) >= 4);
  if (top.length) return top;
  const best = Math.max(...members.map(stars));
  return best > 0 ? members.filter((m) => stars(m) === best) : [members[0]];
}

export type View = {
  stacks: Map<string, Stack>;
  unfolded: Set<string>;
  marks: Marks;
  /** Whether a photo is left out (hidden, and its month not shown with its hidden photos). */
  isHidden: (m: { hash: string; month: string }) => boolean;
};

const visible = (s: Stack, v: View) => s.members.filter((m) => !v.isHidden(m));

/** The month a stack stands in: that of its first visible photo. */
export function home(s: Stack, v: View): string | null {
  return visible(s, v)[0]?.month ?? null;
}

/** The tiles of a stack: folded or all its visible photos. */
export function tilesOf(s: Stack, v: View): Member[] {
  const vis = visible(s, v);
  return v.unfolded.has(s.id) ? vis : folded(vis, v.marks);
}

/** A month's tiles: its (visible) items by date taken, with each stack that stands here together
 * at the place of its first photo, and the photos of stacks standing elsewhere left out. */
export function arrange(month: string, items: Item[], ids: Record<string, string>, v: View): Item[] {
  const out: Item[] = [];
  const placed = new Set<string>();
  for (const item of items) {
    const s = v.stacks.get(ids[item.hash] ?? "");
    if (!s) { out.push(item); continue; }
    if (placed.has(s.id) || home(s, v) !== month) continue;
    placed.add(s.id);
    out.push(...tilesOf(s, v));
  }
  return out;
}

/** How many tiles a month has more (or fewer) than visible photos, because of its stacks; and
 * how many stacks stand there. */
export function stackDelta(month: string, v: View): { delta: number; stacks: number } {
  let delta = 0;
  let stacks = 0;
  for (const s of v.stacks.values()) {
    delta -= visible(s, v).filter((m) => m.month === month).length;
    if (home(s, v) === month) {
      delta += tilesOf(s, v).length;
      stacks++;
    }
  }
  return { delta, stacks };
}

/** Soft colors for stacks: the border, and a light background of the same color (F015/S03). */
const PALETTE = [
  { border: "#60a5fa", bg: "rgba(96, 165, 250, 0.22)" },   // blue
  { border: "#f59e0b", bg: "rgba(245, 158, 11, 0.20)" },   // amber
  { border: "#34d399", bg: "rgba(52, 211, 153, 0.22)" },   // green
  { border: "#f472b6", bg: "rgba(244, 114, 182, 0.20)" },  // pink
  { border: "#a78bfa", bg: "rgba(167, 139, 250, 0.22)" },  // violet
  { border: "#2dd4bf", bg: "rgba(45, 212, 191, 0.22)" },   // teal
  { border: "#fb923c", bg: "rgba(251, 146, 60, 0.20)" },   // orange
  { border: "#94a3b8", bg: "rgba(148, 163, 184, 0.24)" },  // slate
];

export type StackColor = (typeof PALETTE)[number];

/** The same color for the same stack, on every Mac. */
export function stackColor(id: string): StackColor {
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) >>> 0;
  return PALETTE[h % PALETTE.length];
}

/** Whether tile `i` of a month's tiles is the last one shown of its stack, where the symbol goes. */
export function lastOfStack(tiles: { hash: string }[], i: number, ids: Record<string, string>): boolean {
  const id = ids[tiles[i]?.hash];
  return !!id && ids[tiles[i + 1]?.hash ?? ""] !== id;
}

/** What the menu offers for these photos (F015/S04). */
export type StackActions = {
  /** Two or more photos, none in a stack. */
  make: boolean;
  /** Photos of exactly one stack, with others outside any stack: that stack and those others. */
  addTo: { stack: string; photos: string[] } | null;
  /** The photos that are in a stack. */
  takeOut: string[];
  /** The one stack the photos belong to, to dissolve. */
  dissolve: string | null;
};

export function stackActions(hashes: string[], ids: Record<string, string>): StackActions {
  const unique = [...new Set(hashes)];
  const inStack = unique.filter((h) => ids[h]);
  const outside = unique.filter((h) => !ids[h]);
  const stacks = new Set(inStack.map((h) => ids[h]));
  const one = stacks.size === 1 ? [...stacks][0] : null;
  return {
    make: inStack.length === 0 && unique.length >= 2,
    addTo: one && outside.length ? { stack: one, photos: outside } : null,
    takeOut: inStack,
    dissolve: one,
  };
}
