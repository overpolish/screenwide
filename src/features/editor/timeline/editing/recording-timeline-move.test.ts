// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { restoreRecordingTranscriptRanges } from "../../recording/transcript/recording-timeline-transcript";

import {
  recordingTimelineCuts,
  restoreRecordingTimelineCut,
} from "./recording-timeline-cuts";
import {
  clampRecordingTimelineTrimPosition,
  RecordingTimelineEdit,
  recordingTimelineSourceEndToOutput,
  recordingTimelineSourceToOutput,
} from "./recording-timeline-edit";
import {
  moveRecordingTimelineSegment,
  restoreRecordingTimelineOrder,
} from "./recording-timeline-move";
import { recordingTimelineOutputToSourceStart } from "./recording-timeline-source-order";

const DURATION_MS = 10_000;

/** 0-4 s, 4-7 s and 7-10 s of a 10 s recording, uncut. */
const edit = (
  overrides: Partial<RecordingTimelineEdit> = {},
): RecordingTimelineEdit => ({
  artifactId: 1,
  nextSegmentId: 3,
  segments: [
    { id: 0, sourceEnd: 0.4, sourceStart: 0 },
    { id: 1, sourceEnd: 0.7, sourceStart: 0.4 },
    { id: 2, sourceEnd: 1, sourceStart: 0.7 },
  ],
  ...overrides,
});

const move = (
  from: RecordingTimelineEdit,
  segmentId: number,
  boundary: number,
) => moveRecordingTimelineSegment(from, { boundary, segmentId });

describe("moveRecordingTimelineSegment", () => {
  it("moves a segment to the start, between segments and to the end", () => {
    const ids = (next: RecordingTimelineEdit) =>
      next.segments.map((segment) => segment.id);
    expect(ids(move(edit(), 2, 0))).toEqual([2, 0, 1]);
    expect(ids(move(edit(), 0, 2))).toEqual([1, 0, 2]);
    expect(ids(move(edit(), 0, 3))).toEqual([1, 2, 0]);
  });

  it("returns the edit itself for a drop where the segment already is", () => {
    const before = edit();
    expect(move(before, 1, 1)).toBe(before);
    expect(move(before, 1, 2)).toBe(before);
  });
});

describe("restoreRecordingTimelineOrder", () => {
  it("puts the segments back in recorded order, keeping cuts and speeds", () => {
    const reordered = edit({
      segments: [
        { id: 2, playbackRate: 2, sourceEnd: 1, sourceStart: 0.8 },
        { id: 0, sourceEnd: 0.4, sourceStart: 0 },
        { id: 1, sourceEnd: 0.7, sourceStart: 0.5 },
      ],
    });
    expect(restoreRecordingTimelineOrder(reordered).segments).toEqual([
      { id: 0, sourceEnd: 0.4, sourceStart: 0 },
      { id: 1, sourceEnd: 0.7, sourceStart: 0.5 },
      { id: 2, playbackRate: 2, sourceEnd: 1, sourceStart: 0.8 },
    ]);
  });

  it("returns the edit itself where it is already in recorded order", () => {
    const before = edit();
    expect(restoreRecordingTimelineOrder(before)).toBe(before);
  });
});

describe("a reordered timeline", () => {
  // 7-10 s plays first, then 0-4 s, then 4-7 s.
  const reordered = () => move(edit(), 2, 0);

  it("ends a stretch with the segment that ends there", () => {
    // 7 s ends 4-7 s, played last, and starts 7-10 s, played first.
    expect(recordingTimelineSourceToOutput(reordered(), 0.7)).toBe(0);
    expect(recordingTimelineSourceEndToOutput(reordered(), 0.7)).toBe(1);
  });

  it("starts a stretch at a join in the segment after the join", () => {
    expect(
      recordingTimelineOutputToSourceStart(reordered().segments, 0.3),
    ).toBe(0);
  });

  it("lets a trim grow only into source no other segment plays", () => {
    const trimmed = edit({
      segments: [
        { id: 2, sourceEnd: 1, sourceStart: 0.8 },
        { id: 0, sourceEnd: 0.4, sourceStart: 0 },
        { id: 1, sourceEnd: 0.7, sourceStart: 0.5 },
      ],
    });
    const position = (sourcePosition: number, edge: "end" | "start") =>
      clampRecordingTimelineTrimPosition(trimmed, {
        edge,
        minimumDuration: 0,
        segmentId: 1,
        sourcePosition,
      });
    expect(position(0.2, "start")).toBe(0.4);
    expect(position(0.95, "end")).toBe(0.8);
  });

  it("shows and restores a cut after the segment before it in the source", () => {
    const cut = edit({
      segments: [
        { id: 2, sourceEnd: 1, sourceStart: 0.7 },
        { id: 0, sourceEnd: 0.4, sourceStart: 0 },
        { id: 1, sourceEnd: 0.7, sourceStart: 0.5 },
      ],
    });
    const [gap] = recordingTimelineCuts(cut);
    expect(gap.segmentId).toBe(0);
    expect(gap.sourceStart).toBe(0.4);
    expect(gap.sourceEnd).toBe(0.5);
    // After 7-10 s and 0-4 s: 0.7 of the 0.9 kept.
    expect(gap.outputPosition).toBeCloseTo(0.7 / 0.9);
    const restored = restoreRecordingTimelineCut(cut, gap);
    expect(restored.segments).toEqual([
      { id: 2, sourceEnd: 1, sourceStart: 0.7 },
      { id: 0, sourceEnd: 0.7, sourceStart: 0 },
    ]);
  });

  it("brings back a transcript cut beside the segment before it in the source", () => {
    const cut = edit({
      segments: [
        { id: 2, sourceEnd: 1, sourceStart: 0.7 },
        { id: 0, sourceEnd: 0.4, sourceStart: 0 },
        { id: 1, sourceEnd: 0.7, sourceStart: 0.5 },
      ],
    });
    const restored = restoreRecordingTranscriptRanges(
      cut,
      [{ endMs: 5_000, startMs: 4_000 }],
      DURATION_MS,
    );
    expect(restored.segments).toEqual([
      { id: 2, sourceEnd: 1, sourceStart: 0.7 },
      { id: 0, sourceEnd: 0.7, sourceStart: 0 },
    ]);
  });
});
