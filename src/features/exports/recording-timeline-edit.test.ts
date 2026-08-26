// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  clampRecordingTimelineTrimPosition,
  createRecordingTimelineEdit,
  deleteRecordingTimelineRange,
  cutRecordingTimeline,
  deleteRecordingTimelineSegment,
  layoutRecordingTimelineSegments,
  recordingTimelineOutputToSource,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
  remapRecordingTimelinePosition,
  snapRecordingTimelinePosition,
  trimRecordingTimelineSegment,
} from "./recording-timeline-edit";

describe("recording timeline edit", () => {
  it("starts with one range spanning the original recording", () => {
    expect(createRecordingTimelineEdit(42)).toEqual({
      artifactId: 42,
      nextSegmentId: 1,
      segments: [{ id: 0, sourceEnd: 1, sourceStart: 0 }],
    });
  });

  it("splits the containing source range and keeps stable segment ids", () => {
    const first = cutRecordingTimeline(createRecordingTimelineEdit(42), 0.6);
    const second = cutRecordingTimeline(first, 0.2);

    expect(second).toEqual({
      artifactId: 42,
      nextSegmentId: 3,
      segments: [
        { id: 0, sourceEnd: 0.2, sourceStart: 0 },
        { id: 2, sourceEnd: 0.6, sourceStart: 0.2 },
        { id: 1, sourceEnd: 1, sourceStart: 0.6 },
      ],
    });
  });

  it.each([0, 1, 0.5, Number.NaN, Number.POSITIVE_INFINITY])(
    "does not create a duplicate or invalid cut at %s",
    (position) => {
      const edit = cutRecordingTimeline(createRecordingTimelineEdit(42), 0.5);
      expect(cutRecordingTimeline(edit, position)).toBe(edit);
    },
  );
});

describe("recording timeline ripple layout", () => {
  const editWithRemovedMiddle = () => {
    const first = cutRecordingTimeline(createRecordingTimelineEdit(42), 0.2);
    const second = cutRecordingTimeline(first, 0.6);
    return deleteRecordingTimelineSegment(second, 1);
  };

  it("removes a segment and magnetically fills the retained output", () => {
    const edit = editWithRemovedMiddle();

    expect(recordingTimelineRetainedDuration(edit)).toBeCloseTo(0.6);
    const layout = layoutRecordingTimelineSegments(edit);
    expect(
      layout.map(({ id, sourceEnd, sourceStart }) => ({
        id,
        sourceEnd,
        sourceStart,
      })),
    ).toEqual([
      { id: 0, sourceEnd: 0.2, sourceStart: 0 },
      { id: 2, sourceEnd: 1, sourceStart: 0.6 },
    ]);
    expect(layout[0].outputStart).toBe(0);
    expect(layout[0].outputEnd).toBeCloseTo(1 / 3);
    expect(layout[1].outputStart).toBeCloseTo(1 / 3);
    expect(layout[1].outputEnd).toBe(1);
  });

  it("maps positions between retained output time and original source time", () => {
    const edit = editWithRemovedMiddle();

    expect(recordingTimelineOutputToSource(edit, 0.5)).toBeCloseTo(0.7);
    expect(recordingTimelineSourceToOutput(edit, 0.7)).toBeCloseTo(0.5);
    expect(recordingTimelineSourceToOutput(edit, 0.4)).toBeCloseTo(1 / 3);
  });

  it("keeps magnetic time while remapping the source after a deletion", () => {
    const previous = cutRecordingTimeline(
      cutRecordingTimeline(createRecordingTimelineEdit(42), 0.2),
      0.6,
    );
    const next = deleteRecordingTimelineSegment(previous, 1);

    const remapped = remapRecordingTimelinePosition(previous, next, 0.4);
    expect(remapped.outputPosition).toBeCloseTo(2 / 3);
    expect(remapped.sourcePosition).toBeCloseTo(0.8);
  });

  it("clamps a parked playhead to the shortened timeline end", () => {
    const previous = cutRecordingTimeline(
      cutRecordingTimeline(createRecordingTimelineEdit(42), 0.2),
      0.6,
    );
    const next = deleteRecordingTimelineSegment(previous, 1);

    expect(remapRecordingTimelinePosition(previous, next, 0.9)).toEqual({
      outputPosition: 1,
      sourcePosition: 1,
    });
  });

  it("does not delete the last segment or an unknown segment", () => {
    const single = createRecordingTimelineEdit(42);
    expect(deleteRecordingTimelineSegment(single, 0)).toBe(single);
    expect(deleteRecordingTimelineSegment(single, 99)).toBe(single);
  });
});

describe("recording timeline range deletion", () => {
  it("splits at arbitrary boundaries and ripples the remaining ranges", () => {
    const edit = deleteRecordingTimelineRange(
      createRecordingTimelineEdit(42),
      0.2,
      0.6,
    );

    expect(edit).toEqual({
      artifactId: 42,
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.2, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.6 },
      ],
    });
    expect(recordingTimelineRetainedDuration(edit)).toBeCloseTo(0.6);
  });

  it("deletes across existing cuts while preserving unaffected segment ids", () => {
    const cut = cutRecordingTimeline(
      cutRecordingTimeline(createRecordingTimelineEdit(42), 0.2),
      0.6,
    );
    const edit = deleteRecordingTimelineRange(cut, 0.1, 0.8);

    expect(edit.segments).toHaveLength(2);
    expect(edit.segments[0]).toMatchObject({
      id: 0,
      sourceStart: 0,
    });
    expect(edit.segments[0].sourceEnd).toBeCloseTo(0.1);
    expect(edit.segments[1]).toMatchObject({ id: 2, sourceEnd: 1 });
    expect(edit.segments[1].sourceStart).toBeCloseTo(0.8);
    expect(edit.nextSegmentId).toBe(3);
  });

  it("interprets the selection in current magnetic output time", () => {
    const cut = cutRecordingTimeline(
      cutRecordingTimeline(createRecordingTimelineEdit(42), 0.2),
      0.6,
    );
    const withGap = deleteRecordingTimelineSegment(cut, 1);
    const edit = deleteRecordingTimelineRange(withGap, 0.25, 0.75);

    expect(edit.segments).toHaveLength(2);
    expect(edit.segments[0]).toMatchObject({
      id: 0,
      sourceStart: 0,
    });
    expect(edit.segments[0].sourceEnd).toBeCloseTo(0.15);
    expect(edit.segments[1]).toMatchObject({ id: 2, sourceEnd: 1 });
    expect(edit.segments[1].sourceStart).toBeCloseTo(0.85);
  });

  it("normalizes reversed bounds and ignores empty, invalid, or full ranges", () => {
    const initial = createRecordingTimelineEdit(42);
    expect(deleteRecordingTimelineRange(initial, 0.6, 0.2).segments).toEqual([
      { id: 0, sourceEnd: 0.2, sourceStart: 0 },
      { id: 1, sourceEnd: 1, sourceStart: 0.6 },
    ]);
    expect(deleteRecordingTimelineRange(initial, 0.2, 0.2)).toBe(initial);
    expect(deleteRecordingTimelineRange(initial, Number.NaN, 0.5)).toBe(
      initial,
    );
    expect(deleteRecordingTimelineRange(initial, 0, 1)).toBe(initial);
  });
});

describe("recording timeline trimming", () => {
  const cutIntoThirds = () =>
    cutRecordingTimeline(
      cutRecordingTimeline(createRecordingTimelineEdit(42), 0.3),
      0.6,
    );

  it("trims either edge and keeps at least the requested source duration", () => {
    const edit = cutIntoThirds();
    const start = trimRecordingTimelineSegment(edit, {
      edge: "start",
      minimumDuration: 0.1,
      segmentId: 2,
      sourcePosition: 0.8,
    });
    const end = trimRecordingTimelineSegment(edit, {
      edge: "end",
      minimumDuration: 0.1,
      segmentId: 0,
      sourcePosition: 0.05,
    });

    expect(start.segments[2]).toMatchObject({
      id: 2,
      sourceEnd: 1,
      sourceStart: 0.8,
    });
    expect(end.segments[0]).toMatchObject({
      id: 0,
      sourceEnd: 0.1,
      sourceStart: 0,
    });
  });

  it("restores omitted source up to the neighbouring retained segment", () => {
    const cut = cutIntoThirds();
    const deleted = deleteRecordingTimelineSegment(cut, 1);
    const restoredFromLeft = trimRecordingTimelineSegment(deleted, {
      edge: "end",
      minimumDuration: 0.01,
      segmentId: 0,
      sourcePosition: 0.55,
    });
    const restoredFromRight = trimRecordingTimelineSegment(deleted, {
      edge: "start",
      minimumDuration: 0.01,
      segmentId: 2,
      sourcePosition: 0.35,
    });

    expect(restoredFromLeft.segments[0].sourceEnd).toBe(0.55);
    expect(restoredFromRight.segments[1].sourceStart).toBe(0.35);
  });

  it("does not overlap neighbours or rewrite no-op and unknown trims", () => {
    const edit = cutIntoThirds();
    expect(
      trimRecordingTimelineSegment(edit, {
        edge: "end",
        minimumDuration: 0.01,
        segmentId: 0,
        sourcePosition: 0.9,
      }).segments[0].sourceEnd,
    ).toBe(0.3);
    expect(
      trimRecordingTimelineSegment(edit, {
        edge: "start",
        minimumDuration: 0.01,
        segmentId: 99,
        sourcePosition: 0.2,
      }),
    ).toBe(edit);
    expect(
      trimRecordingTimelineSegment(edit, {
        edge: "start",
        minimumDuration: 0.01,
        segmentId: 0,
        sourcePosition: 0,
      }),
    ).toBe(edit);
  });

  it("clamps a trim target to the same bounds the trim itself applies", () => {
    const edit = cutIntoThirds();
    expect(
      clampRecordingTimelineTrimPosition(edit, {
        edge: "end",
        minimumDuration: 0.01,
        segmentId: 0,
        sourcePosition: 0.9,
      }),
    ).toBe(0.3);
    expect(
      clampRecordingTimelineTrimPosition(edit, {
        edge: "start",
        minimumDuration: 0.1,
        segmentId: 1,
        sourcePosition: 0.55,
      }),
    ).toBeCloseTo(0.5);
    expect(
      clampRecordingTimelineTrimPosition(edit, {
        edge: "start",
        minimumDuration: 0.01,
        segmentId: 99,
        sourcePosition: 0.2,
      }),
    ).toBe(0.2);
  });
});

describe("snapRecordingTimelinePosition", () => {
  it("snaps video cuts to the nearest encoded frame", () => {
    expect(snapRecordingTimelinePosition(0.504, 1_000, 30)).toBeCloseTo(
      15 / 30,
    );
    expect(snapRecordingTimelinePosition(0.519, 1_000, 30)).toBeCloseTo(
      16 / 30,
    );
  });

  it("uses millisecond source time for audio-only recordings", () => {
    expect(snapRecordingTimelinePosition(0.5004, 1_000, null)).toBe(0.5);
  });

  it("places whole-second ruler ticks on the nearest fractional-rate frame", () => {
    const snapped = snapRecordingTimelinePosition(
      1_000 / 10_000,
      10_000,
      60_000 / 1_001,
    );

    expect(snapped * 10_000).toBeCloseTo(1_001);
  });

  it("preserves timeline edges even when duration is between frames", () => {
    expect(snapRecordingTimelinePosition(0, 1_015, 30)).toBe(0);
    expect(snapRecordingTimelinePosition(1, 1_015, 30)).toBe(1);
  });
});
