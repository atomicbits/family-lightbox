import { describe, expect, it } from "vitest";
import { messagesText, pushMessage, timeLabel, type Message } from "./messages";
import { fades, type SyncState } from "./sync";

const m = (text: string): Message => ({ at: "2026-10-30T21:15:02+01:00", level: "warn", source: "synchronisatie", text });

describe("messages", () => {
  it("keep the newest first, at most the limit", () => {
    let list: Message[] = [];
    for (const t of ["a", "b", "c"]) list = pushMessage(list, m(t), 2);
    expect(list.map((x) => x.text)).toEqual(["c", "b"]);
  });

  it("copy as the log's lines, oldest first", () => {
    const list = [{ ...m("b"), level: "error" as const }, m("a")];
    expect(messagesText(list)).toBe("2026-10-30T21:15:02+01:00 WARN synchronisatie a\n2026-10-30T21:15:02+01:00 ERROR synchronisatie b\n");
  });

  it("show the time, with the day when it isn't today", () => {
    // in local time, so the test holds in every time zone (the CI runs in UTC)
    const now = new Date(2026, 9, 30, 22, 0, 0);
    expect(timeLabel(new Date(2026, 9, 30, 21, 15, 2).toISOString(), now)).toMatch(/21[:.]15/);
    expect(timeLabel(new Date(2026, 9, 29, 21, 15, 2).toISOString(), now)).toMatch(/29 okt/);
  });
});

describe("which sync lines fade", () => {
  const s = (phase: SyncState["phase"], message: string | null = null) => ({ phase, message }) as SyncState;
  it("done, cancelled and failures go; too little space and the confirmation stay", () => {
    expect(fades(s("done"))).toBe(true);
    expect(fades(s("cancelled"))).toBe(true);
    expect(fades(s("failed", "x.jpg: Operation timed out (os error 60)"))).toBe(true);
    expect(fades(s("failed", "nodig 29,1 GB, vrij 35,2 GB, reserve 20 GB: 13,9 GB tekort"))).toBe(false);
    expect(fades(s("awaiting_confirmation"))).toBe(false);
  });
});
