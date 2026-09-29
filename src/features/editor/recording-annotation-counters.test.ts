// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  RecordingAnnotationClip,
  renumberedAnnotationClips,
} from "./recording-annotations";

const counterClip = (
  id: string,
  value: number,
  startMs: number,
): RecordingAnnotationClip => ({
  annotation: {
    aboveCamera: false,
    animated: true,
    id,
    shape: { angle: 0, center: { x: 10, y: 10 }, kind: "counter", value },
    style: {
      align: "left",
      blur: false,
      color: "#ffcc00",
      handDrawn: false,
      head: "none",
      manual: false,
      radius: 0,
      redaction: "erase",
      softness: 0,
      strength: 0,
      width: 56,
    },
  },
  endMs: startMs + 3_000,
  startMs,
  trackId: "primary",
});

const values = (clips: RecordingAnnotationClip[]) =>
  clips
    .map((clip) => clip.annotation.shape)
    .filter((shape) => shape.kind === "counter")
    .map((shape) => shape.value);

describe("renumberedAnnotationClips", () => {
  it("renumbers counters when a clip is dragged past another", () => {
    const before = [counterClip("a", 1, 1_000), counterClip("b", 2, 4_000)];
    const dragged = [counterClip("a", 1, 6_000), before[1]];
    expect(values(renumberedAnnotationClips(dragged, before))).toEqual([2, 1]);
  });

  it("leaves the numbers alone when only the drawing order changes", () => {
    const before = [counterClip("a", 1, 1_000), counterClip("b", 2, 4_000)];
    const reordered = [before[1], before[0]];
    expect(renumberedAnnotationClips(reordered, before)).toBe(reordered);
  });

  it("keeps tied counters in the order of their numbers, a fresh one last", () => {
    const before = [counterClip("a", 1, 2_000), counterClip("b", 2, 2_000)];
    const drawn = [counterClip("c", 1, 2_000), before[1], before[0]];
    expect(values(renumberedAnnotationClips(drawn, before))).toEqual([3, 2, 1]);
  });

  it("closes the gap a deleted counter leaves", () => {
    const before = [1, 2, 3].map((value) =>
      counterClip(`c${String(value)}`, value, value * 1_000),
    );
    const left = [before[0], before[2]];
    expect(values(renumberedAnnotationClips(left, before))).toEqual([1, 2]);
  });

  it("counts only counters, and leaves arrows where they are", () => {
    const before = [counterClip("c", 1, 5_000)];
    const mixed = [
      before[0],
      {
        ...counterClip("arrow", 0, 0),
        annotation: {
          ...counterClip("arrow", 0, 0).annotation,
          shape: {
            control: { x: 5, y: 5 },
            end: { x: 10, y: 10 },
            kind: "arrow" as const,
            start: { x: 0, y: 0 },
          },
        },
      },
      counterClip("a", 9, 1_000),
    ];
    const numbered = renumberedAnnotationClips(mixed, before);
    expect(values(numbered)).toEqual([2, 1]);
    expect(numbered[1]).toBe(mixed[1]);
  });
});
