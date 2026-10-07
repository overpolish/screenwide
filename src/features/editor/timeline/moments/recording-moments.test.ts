// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";

import {
  adjacentRecordingMoment,
  RecordingMoment,
  visibleRecordingMoments,
} from "./recording-moments";

const moment = (sourceMs: number): RecordingMoment => ({
  color: "#ffcc00",
  index: 0,
  kindId: "funny",
  name: "Funny",
  note: null,
  sourceMs,
});

/** 10 seconds of recording with 4 to 6 seconds cut away. */
const cut: RecordingTimelineEdit = {
  artifactId: 1,
  nextSegmentId: 2,
  segments: [
    { id: 0, sourceEnd: 0.4, sourceStart: 0 },
    { id: 1, sourceEnd: 1, sourceStart: 0.6 },
  ],
};

describe("moments on the cut timeline", () => {
  it("leaves out a moment in a part that was cut away", () => {
    const visible = visibleRecordingMoments(
      cut,
      [moment(2_000), moment(5_000), moment(8_000)],
      10_000,
    );
    expect(visible.map(({ moment }) => moment.sourceMs)).toEqual([
      2_000, 8_000,
    ]);
  });

  it("places a kept moment where its part of the recording now plays", () => {
    const [after] = visibleRecordingMoments(cut, [moment(8_000)], 10_000);
    // 8s is 2s into the second part, which starts 4s into the 8s cut.
    expect(after.output).toBeCloseTo(6 / 8);
  });

  it("follows a part played faster", () => {
    const fast: RecordingTimelineEdit = {
      artifactId: 1,
      nextSegmentId: 2,
      segments: [
        { id: 0, playbackRate: 2, sourceEnd: 0.5, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.5 },
      ],
    };
    const [late] = visibleRecordingMoments(fast, [moment(7_500)], 10_000);
    // The first half plays in 2.5s; 7.5s is 2.5s into the second half.
    expect(late.output).toBeCloseTo(5 / 7.5);
  });

  it("orders moments along the timeline, not by when they were placed", () => {
    const reordered: RecordingTimelineEdit = {
      artifactId: 1,
      nextSegmentId: 2,
      segments: [
        { id: 1, sourceEnd: 1, sourceStart: 0.5 },
        { id: 0, sourceEnd: 0.5, sourceStart: 0 },
      ],
    };
    const visible = visibleRecordingMoments(
      reordered,
      [moment(1_000), moment(9_000)],
      10_000,
    );
    expect(visible.map(({ moment }) => moment.sourceMs)).toEqual([
      9_000, 1_000,
    ]);
  });
});

describe("stepping between moments", () => {
  const visible = visibleRecordingMoments(
    cut,
    [moment(2_000), moment(8_000)],
    10_000,
  );

  it("moves past the moment the playhead stands on", () => {
    expect(adjacentRecordingMoment(visible, visible[0].output, 1)).toBe(
      visible[1].output,
    );
    expect(adjacentRecordingMoment(visible, visible[1].output, -1)).toBe(
      visible[0].output,
    );
  });

  it("finds nothing beyond the first or last moment", () => {
    expect(adjacentRecordingMoment(visible, 0.9, 1)).toBeNull();
    expect(adjacentRecordingMoment(visible, 0.1, -1)).toBeNull();
  });
});
