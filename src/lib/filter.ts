// The filter bar (F002/S03): stars, hearts and kind, kept per Mac; what the Rust core gets to
// filter the index with, and the same test for a list's photos in the frontend.
import type { Item } from "./api";
import type { MarksState } from "./marks";

export type StarsOp = "gte" | "eq" | "lte" | "none";
export type Kind = "photo" | "video" | "live";

export type FilterState = {
  /** n is 1–5; "none" is "without stars". */
  stars: { op: StarsOp; n: number } | null;
  /** `anyone`: any heart; otherwise a heart of one of these people. */
  hearts: { anyone: boolean; people: string[] } | null;
  kinds: Kind[];
  /** Camera names, "" for unknown (S04). */
  cameras: string[];
};

export const NO_FILTER: FilterState = { stars: null, hearts: null, kinds: [], cameras: [] };

export const isSet = (f: FilterState): boolean => !!f.stars || !!f.hearts || f.kinds.length > 0 || f.cameras.length > 0;

/** What the Rust core filters with (`filter::Filter`). */
export type CoreFilter = {
  marks: { hashes: string[]; exclude: boolean } | null;
  kinds: Kind[];
  cameras: string[];
  hidden: string[];
  revealed: string[];
};

function passesStars(stars: number, f: NonNullable<FilterState["stars"]>): boolean {
  switch (f.op) {
    case "gte": return stars >= f.n;
    case "eq": return stars === f.n;
    case "lte": return stars >= 1 && stars <= f.n; // the unrated ones have "zonder sterren"
    case "none": return stars === 0;
  }
}

/** The stars and hearts as a set of photos: those that pass, or, when only "without stars" is
 * asked, those that don't (the rated ones are the short list). */
export function marksSet(f: FilterState, m: Pick<MarksState, "ratings" | "hearts">): CoreFilter["marks"] {
  if (!f.stars && !f.hearts) return null;
  const hearted = (h: string) => {
    const who = Object.keys(m.hearts[h] ?? {});
    return f.hearts!.anyone ? who.length > 0 : who.some((p) => f.hearts!.people.includes(p));
  };
  if (f.stars?.op === "none" && !f.hearts) return { hashes: Object.keys(m.ratings), exclude: true };
  // otherwise only photos with a rating or a heart can pass
  const candidates = f.hearts ? Object.keys(m.hearts) : Object.keys(m.ratings);
  const hashes = candidates.filter((h) =>
    (!f.stars || passesStars(m.ratings[h]?.stars ?? 0, f.stars)) && (!f.hearts || hearted(h)));
  return { hashes, exclude: false };
}

export function toCore(f: FilterState, m: MarksState, revealed: Set<string>): CoreFilter {
  return { marks: marksSet(f, m), kinds: f.kinds, cameras: f.cameras, hidden: Object.keys(m.hidden), revealed: [...revealed] };
}

/** The same test as the core's, for a list's photos, which the frontend has already. */
export function matches(item: Pick<Item, "hash" | "kind" | "live" | "camera">, c: CoreFilter, marksIn?: Set<string>): boolean {
  if (c.marks) {
    const inSet = (marksIn ?? new Set(c.marks.hashes)).has(item.hash);
    if (inSet === c.marks.exclude) return false;
  }
  if (c.kinds.length) {
    const kind: Kind = item.kind === "video" && item.live ? "live" : (item.kind as Kind);
    if (!c.kinds.includes(kind)) return false;
  }
  if (c.cameras.length && !c.cameras.includes(item.camera ?? "")) return false;
  return true;
}

/** A camera's name in the menu; "" is unknown (S04). */
export const cameraLabel = (c: string): string => c || "onbekend";

/** The camera button: "Camera", the camera for one, "3 camera's" for more. */
export function cameraButton(cameras: string[]): string {
  if (cameras.length === 0) return "Camera";
  if (cameras.length === 1) return cameras[0] || "camera onbekend";
  return `${cameras.length} camera's`;
}

/** Ticks or unticks one camera; null is "Alle camera's", which clears the choice. */
export function toggleCamera(cameras: string[], c: string | null): string[] {
  if (c === null) return [];
  return cameras.includes(c) ? cameras.filter((x) => x !== c) : [...cameras, c];
}

const OP: Record<StarsOp, string> = { gte: "≥", eq: "=", lte: "≤", none: "" };
const KIND: Record<Kind, string> = { photo: "foto", video: "video", live: "Live Photo" };

/** "★ ≥ 4 · ♥ lynn · foto, Live Photo", for the bar's tooltip and the toolbar. */
export function filterText(f: FilterState): string {
  const parts: string[] = [];
  if (f.stars) parts.push(f.stars.op === "none" ? "zonder sterren" : `★ ${OP[f.stars.op]} ${f.stars.n}`);
  if (f.hearts) parts.push(f.hearts.anyone ? "♥ iemand" : `♥ ${f.hearts.people.join(", ")}`);
  if (f.kinds.length) parts.push(f.kinds.map((k) => KIND[k]).join(", "));
  if (f.cameras.length) parts.push(f.cameras.length === 1 ? f.cameras[0] || "camera onbekend" : `${f.cameras.length} camera's`);
  return parts.join(" · ");
}

/** A kept filter read back: anything unexpected falls back to no filter. */
export function readFilter(v: unknown): FilterState {
  if (!v || typeof v !== "object") return NO_FILTER;
  const o = v as Partial<FilterState>;
  const stars = o.stars && ["gte", "eq", "lte", "none"].includes(o.stars.op) && o.stars.n >= 0 && o.stars.n <= 5 ? o.stars : null;
  const hearts = o.hearts && Array.isArray(o.hearts.people) ? { anyone: !!o.hearts.anyone, people: o.hearts.people.map(String) } : null;
  const kinds = Array.isArray(o.kinds) ? o.kinds.filter((k): k is Kind => k === "photo" || k === "video" || k === "live") : [];
  const cameras = Array.isArray(o.cameras) ? o.cameras.map(String) : [];
  return { stars, hearts, kinds, cameras };
}
