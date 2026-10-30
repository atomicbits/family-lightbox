import { describe, expect, it } from "vitest";
import { liveTitle } from "./live";

describe("Live Photos", () => {
  it("name the photo a video belongs to", () => {
    expect(liveTitle({ kind: "video", live: "2017/11/iPhone-fotos/2017-11-29-05-11-IMG_0001.HEIC" })).toBe("Live Photo bij 2017-11-29-05-11-IMG_0001.HEIC");
    expect(liveTitle({ kind: "video", live: null })).toBeNull();
    expect(liveTitle({ kind: "photo", live: "x/IMG_0001.HEIC" })).toBeNull();
  });
});
