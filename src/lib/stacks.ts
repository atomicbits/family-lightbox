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
/** The names are those of the marks events (`marks::COLORS` in Rust, F015/S05). */
export const PALETTE = [
  { name: "blauw", border: "#60a5fa", bg: "rgba(96, 165, 250, 0.22)" },
  { name: "amber", border: "#f59e0b", bg: "rgba(245, 158, 11, 0.20)" },
  { name: "groen", border: "#34d399", bg: "rgba(52, 211, 153, 0.22)" },
  { name: "roze", border: "#f472b6", bg: "rgba(244, 114, 182, 0.20)" },
  { name: "paars", border: "#a78bfa", bg: "rgba(167, 139, 250, 0.22)" },
  { name: "turkoois", border: "#2dd4bf", bg: "rgba(45, 212, 191, 0.22)" },
  { name: "oranje", border: "#fb923c", bg: "rgba(251, 146, 60, 0.20)" },
  { name: "grijs", border: "#94a3b8", bg: "rgba(148, 163, 184, 0.24)" },
];

/** A stack's colors: the border, a light translucent background, and the same light color made
 * opaque on the page's background (`Canvas`), to lay over the border where shapes join. */
export type StackColor = (typeof PALETTE)[number] & { fill: string };

const withFill = (c: (typeof PALETTE)[number]): StackColor => ({ ...c, fill: `color-mix(in srgb, ${c.border} 20%, Canvas)` });

/** The color chosen for the stack, or else the same one from its id on every Mac. */
export function stackColor(id: string, chosen?: string): StackColor {
  const picked = chosen ? PALETTE.find((c) => c.name === chosen) : undefined;
  if (picked) return withFill(picked);
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) >>> 0;
  return withFill(PALETTE[h % PALETTE.length]);
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

/** A tile of a stack and which of its neighbors in the grid belong to the same stack, so the
 * stack is drawn as one outline around all its tiles, over several rows (F015, on Peter's request):
 * the background reaches into the gap towards such a neighbor, and a border runs only on the other
 * sides. */
export type Cell = {
  id: string;
  left: boolean; right: boolean; up: boolean; down: boolean;
  upLeft: boolean; upRight: boolean; downLeft: boolean; downRight: boolean;
};

/** Tile `i` of a month's tiles, laid out `cols` to a row. */
export function cell(tiles: { hash: string }[], i: number, cols: number, ids: Record<string, string>): Cell | null {
  const id = ids[tiles[i]?.hash ?? ""];
  if (!id) return null;
  const same = (j: number) => j >= 0 && j < tiles.length && ids[tiles[j].hash] === id;
  const col = i % cols;
  return {
    id,
    left: col > 0 && same(i - 1),
    right: col < cols - 1 && same(i + 1),
    up: same(i - cols),
    down: same(i + cols),
    upLeft: col > 0 && same(i - cols - 1),
    upRight: col < cols - 1 && same(i - cols + 1),
    downLeft: col > 0 && same(i + cols - 1),
    downRight: col < cols - 1 && same(i + cols + 1),
  };
}

/** A rectangle of a stack's shape; `fillet` is a corner of the gap (the side of the square where
 * the arc's center lies) that rounds an inner bend: filled except a quarter circle. */
export type Piece = { left: number; top: number; width: number; height: number; radius: string; fillet?: string };

/** The shape of a stack around one tile at (x, 0), reaching `pad` beyond it: the tile's rectangle
 * (rounded only on corners with no neighbor on either side), a bridge across the gap to a neighbor
 * of the same stack on the right or below, and the gap's corner when four tiles of the stack meet.
 * Drawn once wide in the border color and once at `pad` in the light color on top, the union of all
 * tiles' pieces leaves an even border around the whole stack, inner bends included. */
export function pieces(k: Cell, x: number, tile: number, gap: number, pad: number, r: number, bend = gap): Piece[] {
  const round = (a: boolean, b: boolean) => (a || b ? "0" : `${r}px`);
  const out: Piece[] = [{
    left: x - pad, top: -pad, width: tile + 2 * pad, height: tile + 2 * pad,
    radius: `${round(k.left, k.up)} ${round(k.right, k.up)} ${round(k.right, k.down)} ${round(k.left, k.down)}`,
  }];
  if (k.right) out.push({ left: x + tile, top: -pad, width: gap, height: tile + 2 * pad, radius: "0" });
  if (k.down) out.push({ left: x - pad, top: tile, width: tile + 2 * pad, height: gap, radius: "0" });
  if (k.right && k.down && k.downRight) out.push({ left: x + tile, top: tile, width: gap, height: gap, radius: "0" });
  // inner bends: two neighbors of the stack around a corner whose diagonal isn't one; the arc's center
  // lies `bend` out from the tile in both directions, the same for both layers
  const f = bend - pad;
  if (f > 0) {
    if (k.up && k.left && !k.upLeft) out.push({ left: x - bend, top: -bend, width: f, height: f, radius: "0", fillet: "0 0" });
    if (k.up && k.right && !k.upRight) out.push({ left: x + tile + pad, top: -bend, width: f, height: f, radius: "0", fillet: "100% 0" });
    if (k.down && k.left && !k.downLeft) out.push({ left: x - bend, top: tile + pad, width: f, height: f, radius: "0", fillet: "0 100%" });
    if (k.down && k.right && !k.downRight) out.push({ left: x + tile + pad, top: tile + pad, width: f, height: f, radius: "0", fillet: "100% 100%" });
  }
  return out;
}
