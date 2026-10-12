// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { sceneSpanTarget } from "./recording-scene-span";

const scene = (id: string, startMs: number, endMs: number) => ({
  endMs,
  id,
  startMs,
});

describe("sceneSpanTarget", () => {
  it("adds a scene over the span between scenes, in whole milliseconds", () => {
    expect(
      sceneSpanTarget({
        clips: [scene("a", 0, 1000)],
        lengthMs: 10_000,
        span: { endMs: 3200.6, startMs: 1500.4 },
      }),
    ).toEqual({ endMs: 3201, startMs: 1500, type: "add" });
  });

  it("changes the scene under the span's middle, trimmed to fewer words", () => {
    expect(
      sceneSpanTarget({
        clips: [scene("a", 1000, 4000)],
        lengthMs: 10_000,
        span: { endMs: 3000, startMs: 2000 },
      }),
    ).toEqual({ endMs: 3000, id: "a", startMs: 2000, type: "change" });
  });

  it("stretches a changed scene over more words, up to its neighbours", () => {
    expect(
      sceneSpanTarget({
        clips: [
          scene("a", 0, 1000),
          scene("b", 2000, 3000),
          scene("c", 5000, 6000),
        ],
        lengthMs: 10_000,
        // "b" is the scene the words share most with.
        span: { endMs: 5100, startMs: 800 },
      }),
    ).toEqual({ endMs: 5000, id: "b", startMs: 1000, type: "change" });
  });

  it("changes a scene the words wrap, even off their middle", () => {
    expect(
      sceneSpanTarget({
        clips: [scene("a", 2500, 3000)],
        lengthMs: 10_000,
        span: { endMs: 6400, startMs: 900 },
      }),
    ).toEqual({ endMs: 6400, id: "a", startMs: 900, type: "change" });
  });

  it("keeps a changed scene where it is when its gap is shorter than a scene", () => {
    expect(
      sceneSpanTarget({
        clips: [
          scene("a", 0, 1000),
          scene("b", 1000, 1300),
          scene("c", 1300, 5000),
        ],
        lengthMs: 10_000,
        span: { endMs: 1200, startMs: 1100 },
      }),
    ).toEqual({ endMs: 1300, id: "b", startMs: 1000, type: "change" });
  });

  it("starts where a neighbour ends when the words overlap it by a little", () => {
    expect(
      sceneSpanTarget({
        clips: [scene("a", 0, 2000), scene("b", 5000, 6000)],
        lengthMs: 10_000,
        span: { endMs: 5030, startMs: 1980 },
      }),
    ).toEqual({ endMs: 5000, startMs: 2000, type: "add" });
  });

  it("grows a short span to the shortest scene, centred on it", () => {
    expect(
      sceneSpanTarget({
        clips: [],
        lengthMs: 10_000,
        span: { endMs: 2100, startMs: 1900 },
      }),
    ).toEqual({ endMs: 2250, startMs: 1750, type: "add" });
  });

  it("grows a short span away from a neighbour it would run into", () => {
    expect(
      sceneSpanTarget({
        clips: [scene("a", 0, 2000)],
        lengthMs: 10_000,
        span: { endMs: 2200, startMs: 2000 },
      }),
    ).toEqual({ endMs: 2500, startMs: 2000, type: "add" });
  });

  it("grows a short span back from the end of the video", () => {
    expect(
      sceneSpanTarget({
        clips: [],
        lengthMs: 10_000.4,
        span: { endMs: 10_000, startMs: 9900 },
      }),
    ).toEqual({ endMs: 10_000, startMs: 9500, type: "add" });
  });

  it("is null where the gap around the span is shorter than a scene", () => {
    expect(
      sceneSpanTarget({
        clips: [scene("a", 0, 2000), scene("b", 2400, 6000)],
        lengthMs: 10_000,
        span: { endMs: 2300, startMs: 2100 },
      }),
    ).toBeNull();
  });
});
