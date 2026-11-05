import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { pluginText, type PluginStatus } from "../lib/lightroom";

const button = "px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40";

/** The settings' *Lightroom* (F023/S03): the plug-in that keeps the lists as collections. */
export default function LightroomSection() {
  const [status, setStatus] = useState<PluginStatus | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  useEffect(() => { invoke<PluginStatus>("lightroom_plugin_status").then(setStatus).catch(() => {}); }, []);
  const install = () =>
    invoke<PluginStatus>("lightroom_plugin_install")
      .then((s) => { setStatus(s); setMessage("geïnstalleerd: herstart Lightroom, of kies daar Bestand → Plug-inbeheer → FamilyLightbox → Plug-in opnieuw laden"); })
      .catch((e) => setMessage(String(e)));
  return (
    <section className="space-y-1">
      <div className="flex items-center gap-2">
        <h3 className="font-medium">Lightroom</h3>
        <span className="flex-1" />
        <button className={button} disabled={status === null || status === "current"} onClick={install}>
          {status === "different" ? "Plugin bijwerken" : "Plugin installeren"}
        </button>
      </div>
      <div className="text-neutral-500">{pluginText(status)}</div>
      <div className="text-xs text-neutral-500">
        In Lightroom Classic: Bibliotheek → Plug-in-extra's → Lijsten bijwerken maakt een collectie per lijst, en zet de sterren over (een hartje is 5 sterren).
      </div>
      {message && <div className="text-xs text-neutral-500">{message}</div>}
    </section>
  );
}
