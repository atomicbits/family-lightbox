// The import's sources in the settings (F018/S03): texts and small edits of the settings.
import type { Import, Layout, SourceSettings } from "./api";

export const NO_IMPORT: Import = { drop_folders: [], synced: [], sources: {} };
const NO_SOURCE: SourceSettings = { name: "", exclude: [], ignore: false, delete_imported: false };

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
