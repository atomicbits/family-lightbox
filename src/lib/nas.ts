// One place for the NAS connection state (F006/S05): the header indicator and the settings panel
// both read it from useNas(), so neither calls the commands on its own.
import { useCallback, useRef, useState } from "react";
import { anyMissing, nas, type Attempt, type NasCheck, type NasMounts, type Shares } from "./api";

export type NasSummary = {
  state: "busy" | "connected" | "partial" | "disconnected" | "unreachable";
  label: string;
  tone: "green" | "amber" | "grey";
  action: "connect" | "retry" | null;
};

const KEYS: (keyof Shares)[] = ["library", "previews", "lists", "exports"];

/** What the indicator shows, from the last lookup, check and connect attempts. */
export function nasSummary(
  mounts: NasMounts | null,
  check: NasCheck | null,
  attempts: Attempt[],
  busy: boolean,
): NasSummary {
  if (busy || mounts === null) return { state: "busy", label: "bezig…", tone: "grey", action: null };
  if (mounts.host === "") return { state: "disconnected", label: "niet ingesteld", tone: "grey", action: null };
  if (attempts.some((a) => a.outcome.kind === "unreachable"))
    return { state: "unreachable", label: "NAS niet bereikbaar", tone: "grey", action: "retry" };
  const mounted = KEYS.filter((k) => mounts[k].some((m) => m.chosen)).length;
  if (mounted === 0) return { state: "disconnected", label: "niet verbonden", tone: "grey", action: "connect" };
  const problem = check !== null && KEYS.some((k) => check[k].mounted && check[k].problem !== null);
  const listsWritable = check === null || check.lists.writable !== false;
  if (mounted === KEYS.length && !problem && listsWritable)
    return { state: "connected", label: "verbonden", tone: "green", action: null };
  return { state: "partial", label: "deels verbonden", tone: "amber", action: "connect" };
}

export type Nas = {
  mounts: NasMounts | null;
  check: NasCheck | null;
  attempts: Attempt[];
  busy: boolean;
  error: string | null;
  /** Look up the mounts and check access (which also registers the device). */
  refresh: () => Promise<void>;
  /** Connect what is missing (with the macOS dialog when allowUi), then check. */
  connect: (allowUi: boolean) => Promise<void>;
  /** App start: connect quietly without a dialog if something is missing, then check. Resolves to
   * the mounts found, or null when it didn't run (already busy) or failed. */
  start: () => Promise<NasMounts | null>;
};

export function useNas(): Nas {
  const [mounts, setMounts] = useState<NasMounts | null>(null);
  const [check, setCheck] = useState<NasCheck | null>(null);
  const [attempts, setAttempts] = useState<Attempt[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const running = useRef(false);

  const run = useCallback(async (work: () => Promise<void>) => {
    if (running.current) return;
    running.current = true;
    setBusy(true);
    setError(null);
    try {
      await work();
    } catch (e) {
      setError(String(e));
    } finally {
      running.current = false;
      setBusy(false);
    }
  }, []);

  const checkNow = async () => {
    const c = await nas.check();
    setCheck(c);
    return c;
  };

  const refresh = useCallback(
    () =>
      run(async () => {
        const m = await nas.mounts();
        setMounts(m);
        await checkNow();
        // A stale "unreachable" goes away once everything is mounted again.
        if (!anyMissing(m)) setAttempts([]);
      }),
    [run],
  );

  const connect = useCallback(
    (allowUi: boolean) =>
      run(async () => {
        setAttempts([]);
        const r = await nas.connect(allowUi);
        setMounts(r.mounts);
        setAttempts(r.attempts);
        await checkNow();
      }),
    [run],
  );

  const start = useCallback(async () => {
    let found: NasMounts | null = null;
    await run(async () => {
      let m = await nas.mounts();
      if (anyMissing(m)) {
        // Without UI: succeeds when the Keychain has the account, fails quietly otherwise.
        m = (await nas.connect(false)).mounts;
      }
      setMounts(m);
      await checkNow();
      found = m;
    });
    return found;
  }, [run]);

  return { mounts, check, attempts, busy, error, refresh, connect, start };
}
