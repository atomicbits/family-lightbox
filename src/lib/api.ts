// Brug naar de Rust-kern. Alles wat hier binnenkomt volgt het datacontract (sectie 3, 4 en 7).
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

export type MonthSummary = {
  month: string; // "2026/07", of een mapnaam zonder maand ("2012/scans Peter")
  photos: number;
  videos: number;
  previews: number;
  local: number;
};

export type Item = {
  path: string; // contractpad, relatief aan Library
  folder: string;
  kind: "photo" | "video";
  taken: string | null;
  taken_from: string | null;
  source: string | null;
  size: number;
  hash: string;
  local: boolean;
  preview: boolean;
};

export type Status = {
  index: string;
  index_exists: boolean;
  previews: string;
  previews_exists: boolean;
  library: string;
  library_exists: boolean;
};

export const api = {
  status: () => invoke<Status>("status"),
  months: () => invoke<MonthSummary[]>("months"),
  items: (month: string) => invoke<Item[]>("items", { month }),
  reveal: (path: string) => invoke<void>("reveal", { path }),
  openOriginal: (path: string) => invoke<void>("open_original", { path }),
};

/** preview://localhost/<pad>.jpg of .t.jpg (datacontract 4); convertFileSrc kiest de juiste vorm per platform. */
export function previewUrl(path: string, thumb: boolean): string {
  return convertFileSrc(path + (thumb ? ".t.jpg" : ".jpg"), "preview");
}

export type Shares = { library: string; previews: string; lists: string; exports: string };
export type Settings = { nas_host: string; shares: Shares };
export type SettingsState = { settings: Settings; error: string | null; path: string };
export type Device = { name: string; owner: string };
export type DeviceState = { device: Device | null; error: string | null; path: string };

export const config = {
  settings: () => invoke<SettingsState>("settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  device: () => invoke<DeviceState>("device"),
  createDevice: (name: string, owner: string) => invoke<Device>("create_device", { name, owner }),
};
