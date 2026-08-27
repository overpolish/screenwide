// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingTimelineEdit } from "../recording-timeline-edit";

import {
  layoutTimedLaneItems,
  stackTimedLaneFragments,
  TimedLaneFragment,
  TimedLaneItem,
} from "./timed-lane-layout";

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

describe("stackTimedLaneFragments", () => {
  const fragment = (
    id: string,
    outputStart: number,
    outputEnd: number,
  ): TimedLaneFragment<TimedLaneItem> => ({
    fragmentId: id,
    item: { endMs: outputEnd * 10_000, id, startMs: outputStart * 10_000 },
    outputEnd,
    outputStart,
    segmentId: 0,
  });

  it("keeps non-overlapping fragments in a single row", () => {
    const { fragments, rowCount } = stackTimedLaneFragments([
      fragment("a", 0.0, 0.2),
      fragment("b", 0.2, 0.4),
      fragment("c", 0.6, 0.9),
    ]);
    expect(rowCount).toBe(1);
    expect(fragments.every(({ row }) => row === 0)).toBe(true);
  });

  it("stacks overlapping fragments into sublanes and reuses freed rows", () => {
    const { fragments, rowCount } = stackTimedLaneFragments([
      fragment("fade", 0.0, 0.5),
      fragment("next", 0.3, 0.7),
      fragment("later", 0.55, 0.9),
    ]);
    expect(rowCount).toBe(2);
    const rows = Object.fromEntries(
      fragments.map(({ fragmentId, row }) => [fragmentId, row]),
    );
    expect(rows.fade).toBe(0);
    expect(rows.next).toBe(1);
    // "later" starts after "fade" ended, so it drops back into row zero.
    expect(rows.later).toBe(0);
  });

  it("reports one row for an empty lane", () => {
    expect(stackTimedLaneFragments([]).rowCount).toBe(1);
  });
});
