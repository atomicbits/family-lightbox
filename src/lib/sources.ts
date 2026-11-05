// The import's sources in the settings (F018/S03): texts and small edits of the settings.
import type { Import, Layout, SourceSettings } from "./api";
import { t } from "../i18n";

export const NO_IMPORT: Import = { drop_folders: [], synced: [], sources: {} };
const NO_SOURCE: SourceSettings = { name: "", exclude: [], ignore: false, delete_imported: false, day: null };

/** "met dagmap", "foto's en video's apart, met dagmap", "zonder dagmap". */
export function layoutText(l: Layout): string {
  const tx = t().imports.sources;
  const parts = [l.split ? tx.split : null, l.day ? tx.withDay : tx.withoutDay].filter(Boolean);
  return parts.join(", ");
}

/** The last part of a path, for a folder shown by name. */
export function baseName(path: string): string {
  return path.replace(/\/+$/, "").split("/").pop() ?? path;
}

/** One drop subfolder's settings changed; a setting back to its default is dropped. */
export function withSource(imp: Import, folder: string, change: Partial<SourceSettings>): Import {
  const next = { ...NO_SOURCE, ...imp.sources[folder], ...change };
  const sources = { ...imp.sources };
  if (JSON.stringify(next) === JSON.stringify(NO_SOURCE)) delete sources[folder];
  else sources[folder] = next;
  return { ...imp, sources };
}

/** "a, *-mask.png" → ["a", "*-mask.png"]. */
export function patterns(text: string): string[] {
  return text.split(",").map((p) => p.trim()).filter(Boolean);
}

/** Under a synced folder's sources in the settings: what a source is and where its files go. */
export function syncedHint(photos: string, videos: string): string {
  const tx = t().imports.sources;
  if (!photos.trim() && !videos.trim()) return tx.hintExample("iPhone-photos");
  if (!photos.trim() || !videos.trim()) return tx.hintMissing(!!photos.trim());
  if (photos.trim() === videos.trim()) return tx.hintSame(photos.trim());
  return tx.hintApart(photos.trim(), videos.trim());
}

/** The day-folder choice of a source's settings, for its menu: "last", "day" or "flat". */
export const dayChoice = (day: boolean | null | undefined): "last" | "day" | "flat" => (day == null ? "last" : day ? "day" : "flat");
export const dayFromChoice = (c: string): boolean | null => (c === "day" ? true : c === "flat" ? false : null);
