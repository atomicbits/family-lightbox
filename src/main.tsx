import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { config } from "./lib/api";
import { LANGUAGE_EVENT, LanguageContext, pickLanguage, setLanguage, type Lang } from "./i18n";
import { invoke } from "@tauri-apps/api/core";
import "./index.css";

/** Keeps the language in use: the Mac's at first, then the setting's; a change re-renders all. */
function Root() {
  const [lang, setLang] = useState<Lang>(() => {
    const l = pickLanguage("", navigator.languages);
    setLanguage(l);
    return l;
  });
  useEffect(() => {
    const apply = (setting: string | undefined) => {
      const l = pickLanguage(setting, navigator.languages);
      setLanguage(l);
      setLang(l);
    };
    config.settings().then((s) => apply(s.settings.language)).catch(() => {});
    const on = (e: Event) => apply((e as CustomEvent<string>).detail);
    window.addEventListener(LANGUAGE_EVENT, on);
    return () => window.removeEventListener(LANGUAGE_EVENT, on);
  }, []);
  useEffect(() => {
    document.documentElement.lang = lang;
    // the core's messages in the same language (F011/S03)
    invoke("set_language", { lang }).catch(() => {});
  }, [lang]);
  return (
    <LanguageContext.Provider value={lang}>
      <App />
    </LanguageContext.Provider>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
