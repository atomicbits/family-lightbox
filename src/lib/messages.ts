// Messages (F016/S02): the warnings and errors of this session, from the Rust core's `message`
// events (its own and those the UI logs through `log_message`), newest first, behind the ⚠ in
// the footer.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Message = { at: string; level: "info" | "warn" | "error"; source: string; text: string };

export const MAX_MESSAGES = 500;

/** The list with a new message on top, at most `max`. */
export function pushMessage(list: Message[], m: Message, max = MAX_MESSAGES): Message[] {
  return [m, ...list].slice(0, max);
}

/** The messages as lines of text, oldest first, as in the log file: to copy and send on. */
export function messagesText(list: Message[]): string {
  return [...list].reverse().map((m) => `${m.at} ${m.level.toUpperCase()} ${m.source} ${m.text}`).join("\n") + "\n";
}

/** "21:15" for today, "30 okt 21:15" for an earlier day. */
export function timeLabel(at: string, now = new Date()): string {
  const d = new Date(at);
  if (Number.isNaN(d.getTime())) return at;
  const time = d.toLocaleTimeString("nl-BE", { hour: "2-digit", minute: "2-digit" });
  return d.toDateString() === now.toDateString() ? time : `${d.toLocaleDateString("nl-BE", { day: "numeric", month: "short" })} ${time}`;
}

export type Messages = {
  list: Message[];
  /** How many came in since the list was last opened. */
  unseen: number;
  seen: () => void;
  clear: () => void;
  logPath: string;
};

export function useMessages(): Messages {
  const [list, setList] = useState<Message[]>([]);
  const [unseen, setUnseen] = useState(0);
  const [logPath, setLogPath] = useState("");

  useEffect(() => {
    invoke<string>("log_path").then(setLogPath).catch(() => {});
    const off = listen<Message>("message", (e) => {
      setList((l) => pushMessage(l, e.payload));
      setUnseen((n) => n + 1);
    });
    return () => { off.then((f) => f()); };
  }, []);

  const seen = useCallback(() => setUnseen(0), []);
  const clear = useCallback(() => { setList([]); setUnseen(0); }, []);
  return { list, unseen, seen, clear, logPath };
}
