// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "../../annotations/annotations";

import { annotationGlides, glideMs } from "./recording-annotation-glides";
import { RecordingAnnotationClip } from "./recording-annotations";

import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

const arrow = (x: number): Annotation["shape"] => ({
  control: { x: x + 50, y: 0 },
  end: { x: x + 100, y: 0 },
  kind: "arrow",
  start: { x, y: 0 },
});

const placed = (places: Record<string, number>): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id: "arrow-1",
    shape: arrow(0),
    style: {
      align: "left",
      blur: false,
      color: "#ff0000",
      handDrawn: false,
      head: "end",
      manual: false,
      radius: 0,
      redaction: "erase",
      shadow: false,
      softness: 0,
      strength: 0,
      tint: false,
      width: 8,
    },
  },
  endMs: 20_000,
  sceneShapes: Object.entries(places).map(([sceneId, x]) => ({
    sceneId,
    shape: arrow(x),
  })),
  startMs: 0,
  trackId: "primary",
});

const scene = (id: string, startMs: number, endMs: number) =>
  ({ endMs, id, preset: "screen-only", startMs }) as RecordingSceneClip;

const picture = { height: 1_000, width: 1_000 };

describe("glideMs", () => {
  it("paces a move as its twin in glide.rs does", () => {
    // The same case as `a_glide_takes_as_long_as_its_twin_on_the_strip_says`.
    expect(glideMs(arrow(0), arrow(1_300), picture)).toBeCloseTo(1_150.5, 0);
    expect(glideMs(arrow(0), arrow(100), picture)).toBe(600);
    expect(glideMs(arrow(0), arrow(10_000), picture)).toBe(1_500);
  });
});

describe("annotationGlides", () => {
  it("arrives as its scene does and leaves before it ends", () => {
    const glides = annotationGlides({
      clip: placed({ s: 1_300 }),
      picture,
      scenes: [scene("s", 4_000, 10_000)],
      videoMs: 30_000,
    });
    expect(glides).toHaveLength(2);
    expect(glides[0]?.startMs).toBe(4_000);
    expect(glides[0]?.endMs).toBeCloseTo(5_150.5, 0);
    expect(glides[1]?.startMs).toBeCloseTo(8_849.5, 0);
    expect(glides[1]?.endMs).toBe(10_000);
  });

  it("hands straight on between butting scenes, and home from one with no place", () => {
    const scenes = [scene("one", 2_000, 6_000), scene("two", 6_000, 12_000)];
    expect(
      annotationGlides({
        clip: placed({ one: 100, two: 200 }),
        picture,
        scenes,
        videoMs: 30_000,
      }),
    ).toEqual([
      { endMs: 2_600, startMs: 2_000 },
      { endMs: 6_600, startMs: 6_000 },
      { endMs: 12_000, startMs: 11_400 },
    ]);
    expect(
      annotationGlides({
        clip: placed({ one: 100 }),
        picture,
        scenes,
        videoMs: 30_000,
      }),
    ).toEqual([
      { endMs: 2_600, startMs: 2_000 },
      { endMs: 6_600, startMs: 6_000 },
    ]);
  });

  it("takes no time where a scene holds at an end of the video", () => {
    expect(
      annotationGlides({
        clip: placed({ s: 100 }),
        picture,
        scenes: [scene("s", 0, 5_000)],
        videoMs: 30_000,
      }),
    ).toEqual([{ endMs: 5_000, startMs: 4_400 }]);
  });

  it("has nothing to travel where it appears and goes inside its scene", () => {
    expect(
      annotationGlides({
        clip: { ...placed({ s: 1_300 }), endMs: 9_900, startMs: 4_200 },
        picture,
        scenes: [scene("s", 4_000, 10_000)],
        videoMs: 30_000,
      }),
    ).toEqual([]);
  });
});
