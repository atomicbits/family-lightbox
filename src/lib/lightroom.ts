// The Lightroom plug-in in the settings (F023/S03).
import { t } from "../i18n";

export type PluginStatus = "missing" | "current" | "different";

export function pluginText(s: PluginStatus | null): string {
  if (s === null) return t().settings.lightroom.checking;
  if (s === "current") return t().settings.lightroom.current;
  if (s === "different") return t().settings.lightroom.different;
  return t().settings.lightroom.missing;
}
