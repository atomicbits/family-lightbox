import { describe, expect, it } from "vitest";
import { previewsLine, type MakeState } from "./previews";

const base: MakeState = {
  running: false, paused: false, total: 0, done: 0, made: 0, copied: 0, failed: 0, skipped: 0, nas_failed: 0, message: null,
};

describe("previews line", () => {
  it("shows the progress with Pauzeren while making", () => {
    expect(previewsLine({ ...base, running: true, total: 85874, done: 1204 })).toEqual({
      text: "Previews maken: 1 204 / 85 874", tone: "muted", button: { label: "Pauzeren", action: "pause" },
    });
  });

  it("shows Hervatten when paused, with or without a run", () => {
    expect(previewsLine({ ...base, paused: true, total: 85874, done: 1204 })?.text).toBe("Previews gepauzeerd: 1 204 / 85 874");
    expect(previewsLine({ ...base, paused: true })).toMatchObject({ text: "Previews gepauzeerd", button: { action: "resume" } });
  });

  it("says nothing while planning or when all is done, and warns when the NAS didn't get them", () => {
    expect(previewsLine({ ...base, running: true })).toBeNull();
    expect(previewsLine({ ...base, total: 10, done: 10, made: 10 })).toBeNull();
    expect(previewsLine({ ...base, total: 10, done: 10, made: 10, nas_failed: 3 })).toMatchObject({
      text: "3 previews staan nog niet op de NAS", tone: "amber",
    });
    expect(previewsLine({ ...base, nas_failed: 1 })?.text).toBe("1 preview staat nog niet op de NAS");
    expect(previewsLine(null)).toBeNull();
  });
});
