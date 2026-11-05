// The app's languages (F011): English, the reference, and Dutch. The language in use is kept here,
// so the helpers that build texts outside React read it too; the root's context re-renders the
// window when it changes. Numbers and month names follow it.
import { createContext, useContext } from "react";
import { en, type Catalog } from "./en";
import { nl } from "./nl";

export type Lang = "en" | "nl";
/** The setting: empty follows the Mac. */
export type LangSetting = "" | Lang;

const CATALOGS: Record<Lang, Catalog> = { en, nl };

let current: Lang = "en";

/** Dutch when the Mac's first preferred language is Dutch, English otherwise; a setting wins. */
export function pickLanguage(setting: string | undefined, preferred: readonly string[]): Lang {
  if (setting === "en" || setting === "nl") return setting;
  return (preferred[0] ?? "").toLowerCase().split("-")[0] === "nl" ? "nl" : "en";
}

export function setLanguage(lang: Lang): void {
  current = lang;
}

export function language(): Lang {
  return current;
}

/** The texts in the language in use. */
export function t(): Catalog {
  return CATALOGS[current];
}

/** 124837 → "124,837" in English, "124 837" in Dutch. */
export function count(n: number): string {
  const s = String(n);
  return s.replace(/\B(?=(\d{3})+(?!\d))/g, current === "nl" ? " " : ",");
}

/** A number with `digits` decimals: "1.3" / "1,3". */
export function decimal(n: number, digits = 1): string {
  const s = n.toFixed(digits);
  return current === "nl" ? s.replace(".", ",") : s;
}

/** The month's name, 1 to 12: "July" / "juli". */
export function monthName(m: number): string {
  return t().months[m - 1] ?? String(m);
}

/** Sent on `window` with the new setting when the settings change the language. */
export const LANGUAGE_EVENT = "familylightbox-language";

/** Re-renders whatever reads it when the language changes. */
export const LanguageContext = createContext<Lang>("en");

export function useLanguage(): Lang {
  return useContext(LanguageContext);
}
