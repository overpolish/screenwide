// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  recordingTimelineCuts,
  restoreRecordingTimelineCut,
} from "./recording-timeline-cuts";
import { RecordingTimelineEdit } from "./recording-timeline-edit";

/** Three segments: 0.4 to 0.5 cut out, then a plain split at 0.8. */
const edit = (
  overrides: Partial<RecordingTimelineEdit> = {},
): RecordingTimelineEdit => ({
  artifactId: 1,
  nextSegmentId: 3,
  segments: [
    { id: 0, sourceEnd: 0.4, sourceStart: 0 },
    { id: 1, sourceEnd: 0.8, sourceStart: 0.5 },
    { id: 2, sourceEnd: 1, sourceStart: 0.8 },
  ],
  ...overrides,
});

describe("recordingTimelineCuts", () => {
  it("finds the join where source was left out, not a plain split", () => {
    const cuts = recordingTimelineCuts(edit());
    expect(cuts).toHaveLength(1);
    expect(cuts[0].segmentId).toBe(0);
    expect(cuts[0].sourceStart).toBe(0.4);
    expect(cuts[0].sourceEnd).toBe(0.5);
    // 0.4 of the 0.9 still kept plays before the join.
    expect(cuts[0].outputPosition).toBeCloseTo(0.4 / 0.9);
  });

  it("finds what was trimmed off either end of the recording", () => {
    const trimmed = edit({
      segments: [
        { id: 0, sourceEnd: 0.4, sourceStart: 0.1 },
        { id: 1, sourceEnd: 0.9, sourceStart: 0.4 },
      ],
    });
    expect(recordingTimelineCuts(trimmed)).toEqual([
      {
        followingSegmentId: 0,
        outputPosition: 0,
        segmentId: null,
        sourceEnd: 0.1,
        sourceStart: 0,
      },
      {
        followingSegmentId: null,
        outputPosition: 1,
        segmentId: 1,
        sourceEnd: 1,
        sourceStart: 0.9,
      },
    ]);
  });
});

const join = { followingSegmentId: 1, segmentId: 0 };

describe("restoreRecordingTimelineCut", () => {
  it("rejoins neighbours that play at the same rate", () => {
    expect(restoreRecordingTimelineCut(edit(), join).segments).toEqual([
      { id: 0, sourceEnd: 0.8, sourceStart: 0 },
      { id: 2, sourceEnd: 1, sourceStart: 0.8 },
    ]);
  });

  it("grows the earlier segment when the rates differ", () => {
    const sped = edit();
    sped.segments[1] = { ...sped.segments[1], playbackRate: 2 };
    expect(
      restoreRecordingTimelineCut(sped, join).segments.slice(0, 2),
    ).toEqual([
      { id: 0, sourceEnd: 0.5, sourceStart: 0 },
      { id: 1, playbackRate: 2, sourceEnd: 0.8, sourceStart: 0.5 },
    ]);
  });

  it("keeps a segment carrying a shortcut edit apart", () => {
    const moved = edit({
      deletedKeyboardShortcutFragments: [{ segmentId: 1, shortcutId: 7 }],
    });
    const restored = restoreRecordingTimelineCut(moved, join);
    expect(restored.segments.map((segment) => segment.id)).toEqual([0, 1, 2]);
    expect(restored.segments[0].sourceEnd).toBe(0.5);
  });

  it("reaches the first and last segments back to the recording's ends", () => {
    const trimmed = edit({
      segments: [
        { id: 0, playbackRate: 2, sourceEnd: 0.4, sourceStart: 0.1 },
        { id: 1, sourceEnd: 0.9, sourceStart: 0.4 },
      ],
    });
    const start = restoreRecordingTimelineCut(trimmed, {
      followingSegmentId: 0,
      segmentId: null,
    });
    expect(start.segments[0]).toEqual({
      id: 0,
      playbackRate: 2,
      sourceEnd: 0.4,
      sourceStart: 0,
    });
    const end = restoreRecordingTimelineCut(start, {
      followingSegmentId: null,
      segmentId: 1,
    });
    expect(end.segments[1]).toEqual({ id: 1, sourceEnd: 1, sourceStart: 0.4 });
  });

  it("returns the same edit where nothing is cut", () => {
    const before = edit();
    for (const cut of [
      { followingSegmentId: 2, segmentId: 1 },
      { followingSegmentId: 0, segmentId: null },
      { followingSegmentId: null, segmentId: 2 },
      { followingSegmentId: null, segmentId: 1 },
    ]) {
      expect(restoreRecordingTimelineCut(before, cut)).toBe(before);
    }
  });
});
