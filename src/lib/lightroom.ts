// The Lightroom plug-in in the settings (F023/S03).
export type PluginStatus = "missing" | "current" | "different";

export function pluginText(s: PluginStatus | null): string {
  if (s === null) return "nakijken…";
  if (s === "current") return "de plugin van FamilyLightbox is geïnstalleerd";
  if (s === "different") return "er staat een andere versie van de plugin: werk ze bij";
  return "de plugin is nog niet geïnstalleerd";
}
