// Updates in the UI (F004/S04): one check at start, *Bijwerken* in the header, and the *Over*
// section in the settings. The Rust side downloads, checks the signature, installs and restarts.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";

export type Offer = { version: string; notes: string };
export type Progress = { done: number; total: number | null };

export type UpdateState = {
  /** The newer version found, if any. */
  offer: Offer | null;
  /** Downloading and installing; the app restarts when it is done. */
  progress: Progress | null;
  installing: boolean;
  /** What went wrong installing: shown in the header. */
  installError: string | null;
  /** What went wrong in a check by hand: shown in the settings. */
  checkError: string | null;
  /** The last check that was asked for: "Je hebt de nieuwste versie" or the version found. */
  checked: string | null;
  checking: boolean;
};

export const initial: UpdateState = {
  offer: null,
  progress: null,
  installing: false,
  installError: null,
  checkError: null,
  checked: null,
  checking: false,
};

export const UP_TO_DATE = "Je hebt de nieuwste versie";

export type UpdateLine = {
  text: string;
  button: { label: string; disabled: boolean; title: string } | null;
  tone: "muted" | "amber" | "red";
};

function percent(p: Progress): string {
  return p.total ? ` ${Math.min(100, Math.floor((p.done / p.total) * 100))} %` : "";
}

/** The header: nothing until there is a newer version, an install running, or an install failed. */
export function updateLine(s: UpdateState, syncing: boolean): UpdateLine | null {
  if (s.installing) return { text: `Bijwerken…${s.progress ? percent(s.progress) : ""}`, button: null, tone: "muted" };
  if (!s.offer) return s.installError ? { text: s.installError, button: null, tone: "red" } : null;
  // a failed download keeps the offer, so the button stays for another try
  return {
    text: s.installError ?? `Versie ${s.offer.version} beschikbaar`,
    button: {
      label: "Bijwerken",
      disabled: syncing,
      title: syncing ? "na de synchronisatie" : `FamilyLightbox ${s.offer.version} installeren en herstarten`,
    },
    tone: s.installError ? "red" : "amber",
  };
}

/** What a check by hand reports. */
export function checkedText(offer: Offer | null): string {
  return offer ? `Versie ${offer.version} is beschikbaar` : UP_TO_DATE;
}

export type Update = {
  state: UpdateState;
  version: string | null;
  /** `byHand` reports the outcome and errors in the settings; at start a failure stays silent. */
  check: (byHand: boolean) => Promise<void>;
  install: () => Promise<void>;
};

export function useUpdate(): Update {
  const [state, setState] = useState<UpdateState>(initial);
  const [version, setVersion] = useState<string | null>(null);

  const check = useCallback(async (byHand: boolean) => {
    setState((s) => ({ ...s, checking: true, ...(byHand ? { checkError: null, checked: null } : {}) }));
    try {
      const offer = await invoke<Offer | null>("update_check");
      setState((s) => ({ ...s, checking: false, offer, ...(byHand ? { checked: checkedText(offer) } : {}) }));
    } catch (e) {
      // no network or no release yet: not worth an error for an update nobody asked for
      console.warn("update check:", e);
      setState((s) => ({ ...s, checking: false, ...(byHand ? { checkError: String(e) } : {}) }));
    }
  }, []);

  const install = useCallback(async () => {
    setState((s) => ({ ...s, installing: true, progress: null, installError: null }));
    try {
      await invoke("update_install"); // restarts the app when it succeeds
    } catch (e) {
      // a refused signature drops the offer on the Rust side too
      const refused = String(e).startsWith("Update geweigerd");
      setState((s) => ({ ...s, installing: false, progress: null, installError: String(e), offer: refused ? null : s.offer }));
    }
  }, []);

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
    const un = listen<Progress>("update-progress", (e) => setState((s) => ({ ...s, progress: e.payload })));
    // once, a little after start, and never in a dev build
    const timer = import.meta.env.DEV ? null : setTimeout(() => check(false), 5000);
    return () => {
      if (timer) clearTimeout(timer);
      un.then((f) => f());
    };
  }, [check]);

  return { state, version, check, install };
}
