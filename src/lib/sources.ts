// The import's sources in the settings (F018/S03): texts and small edits of the settings.
import type { Import, Layout, SourceSettings } from "./api";

export const NO_IMPORT: Import = { drop_folders: [], synced: [], sources: {} };
const NO_SOURCE: SourceSettings = { name: "", exclude: [], ignore: false, delete_imported: false, day: null };

/** "met dagmap", "foto's en video's apart, met dagmap", "zonder dagmap". */
export function layoutText(l: Layout): string {
  const parts = [l.split ? "foto's en video's apart" : null, l.day ? "met dagmap" : "zonder dagmap"].filter(Boolean);
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
  const at = (src: string) => `<jaar>/<maand>/${src.trim()}/`;
  const what = "Een bron is de map in de bibliotheek waarin de bestanden uit deze map komen, per maand";
  if (!photos.trim() && !videos.trim()) return `${what}, bv. ${at("iPhone-photos")}. Foto's en video's mogen dezelfde bron hebben.`;
  if (!photos.trim() || !videos.trim()) return `${what}. Geef ook de bron voor de ${photos.trim() ? "video's" : "foto's"}; dat mag dezelfde zijn.`;
  if (photos.trim() === videos.trim()) return `Foto's en video's komen samen in ${at(photos)}.`;
  return `Foto's komen in ${at(photos)}, video's in ${at(videos)}.`;
}

/** The day-folder choice of a source's settings, for its menu: "last", "day" or "flat". */
export const dayChoice = (day: boolean | null | undefined): "last" | "day" | "flat" => (day == null ? "last" : day ? "day" : "flat");
export const dayFromChoice = (c: string): boolean | null => (c === "day" ? true : c === "flat" ? false : null);
