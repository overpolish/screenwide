// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
  deleteRecordingTimelineSegment,
} from "../editing/recording-timeline-edit";

import {
  moveTimedSpanClip,
  resizeTimedSpanClip,
  TimedSpanClip,
} from "./timed-span-clips";

const DURATION = 60_000;
const MINIMUM_MS = 500;
const clip = (id: string, startMs: number, endMs: number): TimedSpanClip => ({
  endMs,
  id,
  startMs,
});

describe("moveTimedSpanClip", () => {
  const edit = createRecordingTimelineEdit(1);

  const move = (clips: TimedSpanClip[], id: string, deltaMs: number) =>
    moveTimedSpanClip({
      clips,
      deltaOutput: deltaMs / DURATION,
      edit,
      id,
      minimumMs: MINIMUM_MS,
      sourceDurationMs: DURATION,
    });

  it("passes over a neighbour into the gap beyond, keeping its length", () => {
    const moved = move(
      [clip("a", 0, 5_000), clip("b", 10_000, 20_000)],
      "a",
      20_000,
    );
    expect(moved.map((item) => item.id)).toEqual(["b", "a"]);
    expect(moved[1]).toMatchObject({ endMs: 25_000, startMs: 20_000 });
  });

  it("fills a gap shorter than itself", () => {
    // b, 14s long, is dropped with its middle in the 2s between a and c.
    const moved = move(
      [
        clip("a", 0, 4_000),
        clip("c", 6_000, 10_000),
        clip("b", 20_000, 34_000),
      ],
      "b",
      -22_000,
    );
    expect(moved.map((item) => item.id)).toEqual(["a", "b", "c"]);
    expect(moved[1]).toMatchObject({ endMs: 6_000, startMs: 4_000 });
  });

  it("passes over a gap shorter than the minimum", () => {
    // The 400ms between a and c cannot hold a clip, so b stays put.
    const clips = [
      clip("a", 0, 4_000),
      clip("c", 4_400, 10_000),
      clip("b", 20_000, 25_000),
    ];
    expect(move(clips, "b", -18_000)[2]).toMatchObject({
      endMs: 15_000,
      startMs: 10_000,
    });
  });

  it("stops at the ends of the recording", () => {
    expect(move([clip("a", 10_000, 15_000)], "a", -60_000)[0]).toMatchObject({
      endMs: 5_000,
      startMs: 0,
    });
  });

  it("stops at the end of the edited video, not of the recording", () => {
    // Cut 20s-30s out: the video is 50s long.
    const cut = cutRecordingTimeline(
      cutRecordingTimeline(edit, 20_000 / DURATION),
      30_000 / DURATION,
    );
    const removed = deleteRecordingTimelineSegment(cut, cut.segments[1].id);
    const moved = moveTimedSpanClip({
      clips: [clip("a", 15_000, 20_000)],
      deltaOutput: 1,
      edit: removed,
      id: "a",
      minimumMs: MINIMUM_MS,
      sourceDurationMs: DURATION,
    });
    expect(moved[0]).toMatchObject({ endMs: 50_000, startMs: 45_000 });
  });
});

describe("resizeTimedSpanClip", () => {
  const edit = createRecordingTimelineEdit(1);
  const clips = [clip("a", 0, 5_000), clip("b", 10_000, 15_000)];

  it("trims no further than the neighbour", () => {
    const trimmed = resizeTimedSpanClip({
      clips,
      edge: "startMs",
      edit,
      id: "b",
      minimumMs: MINIMUM_MS,
      output: 0,
      sourceDurationMs: DURATION,
    });
    expect(trimmed[1].startMs).toBe(5_000);
  });

  it("keeps the shortest a clip may be", () => {
    const trimmed = resizeTimedSpanClip({
      clips,
      edge: "endMs",
      edit,
      id: "b",
      minimumMs: MINIMUM_MS,
      output: 0,
      sourceDurationMs: DURATION,
    });
    expect(trimmed[1]).toMatchObject({ endMs: 10_500, startMs: 10_000 });
  });
});
