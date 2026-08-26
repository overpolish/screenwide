// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingTimelineEdit } from "../recording-timeline-edit";

import { layoutTimedLaneItems } from "./timed-lane-layout";

const edit: RecordingTimelineEdit = {
  artifactId: 1,
  nextSegmentId: 2,
  segments: [
    { id: 0, sourceEnd: 0.4, sourceStart: 0 },
    { id: 1, sourceEnd: 1, sourceStart: 0.6 },
  ],
};

describe("timed lane layout", () => {
  it("removes items inside a timeline cut and magnetically maps retained items", () => {
    const fragments = layoutTimedLaneItems({
      edit,
      items: [
        { endMs: 2_000, id: "before", startMs: 1_000 },
        { endMs: 5_500, id: "cut", startMs: 5_000 },
        { endMs: 8_000, id: "after", startMs: 7_000 },
      ],
      sourceDurationMs: 10_000,
    });

    expect(fragments.map(({ item }) => item.id)).toEqual(["before", "after"]);
    expect(fragments[1]?.outputStart).toBeCloseTo(0.625);
  });

  it("splits a ranged item across removed source time", () => {
    const fragments = layoutTimedLaneItems({
      edit,
      items: [{ endMs: 7_000, id: "caption", startMs: 3_000 }],
      sourceDurationMs: 10_000,
    });

    expect(fragments).toHaveLength(2);
    expect(fragments.map(({ fragmentId }) => fragmentId)).toEqual([
      "caption:0",
      "caption:1",
    ]);
    expect(fragments[0]?.outputStart).toBeCloseTo(0.375);
    expect(fragments[0]?.outputEnd).toBeCloseTo(0.5);
    expect(fragments[1]?.outputStart).toBeCloseTo(0.5);
    expect(fragments[1]?.outputEnd).toBeCloseTo(0.625);
  });

  it("keeps zero-duration shortcut events visible", () => {
    const fragments = layoutTimedLaneItems({
      edit,
      items: [{ endMs: 2_000, id: 7, startMs: 2_000 }],
      sourceDurationMs: 10_000,
    });

    expect(fragments).toHaveLength(1);
    expect(fragments[0]?.outputStart).toBe(fragments[0]?.outputEnd);
  });
});
