// Live Photos (F017/S02): the text for a video that is the moving part of a photo.
import type { Item } from "./api";
import { t } from "../i18n";

/** "Live Photo bij IMG_1234.HEIC", or null for a video that isn't one. */
export function liveTitle(item: Pick<Item, "kind" | "live">): string | null {
  if (item.kind !== "video" || !item.live) return null;
  return t().browse.live.title(item.live.split("/").pop() ?? "");
}
