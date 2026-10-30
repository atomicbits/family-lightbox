// Live Photos (F017/S02): the text for a video that is the moving part of a photo.
import type { Item } from "./api";

/** "Live Photo bij IMG_1234.HEIC", or null for a video that isn't one. */
export function liveTitle(item: Pick<Item, "kind" | "live">): string | null {
  if (item.kind !== "video" || !item.live) return null;
  return `Live Photo bij ${item.live.split("/").pop()}`;
}
