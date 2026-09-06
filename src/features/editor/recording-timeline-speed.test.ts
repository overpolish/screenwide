// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  deletedRecordingKeyboardShortcutFragments,
  recordingKeyboardShortcutPositions,
} from "./recording-keyboard-timeline-edit";
import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
  recordingTimelineRetainedDuration,
  setRecordingTimelineSegmentPlaybackRate,
} from "./recording-timeline-edit";
import {
  recordingTimelineRangePlaybackRate,
  setRecordingTimelineRangePlaybackRate,
} from "./recording-timeline-speed";

describe("recording timeline range speed", () => {
  it("splits one segment at both selected output boundaries", () => {
    const edit = setRecordingTimelineRangePlaybackRate(
      createRecordingTimelineEdit(42),
      { outputEnd: 0.75, outputStart: 0.25, playbackRate: 2 },
    );

    expect(edit).toEqual({
      artifactId: 42,
      nextSegmentId: 3,
      segments: [
        { id: 0, sourceEnd: 0.25, sourceStart: 0 },
        {
          id: 1,
          playbackRate: 2,
          sourceEnd: 0.75,
          sourceStart: 0.25,
        },
        { id: 2, sourceEnd: 1, sourceStart: 0.75 },
      ],
    });
    expect(recordingTimelineRetainedDuration(edit)).toBe(0.75);
  });

  it("maps a cross-segment selection through each existing rate", () => {
    const cut = cutRecordingTimeline(createRecordingTimelineEdit(42), 0.5);
    const mixed = setRecordingTimelineSegmentPlaybackRate(cut, 1, 0.5);
    expect(recordingTimelineRangePlaybackRate(mixed, 0, 1)).toBeUndefined();

    const edit = setRecordingTimelineRangePlaybackRate(mixed, {
      outputEnd: 2 / 3,
      outputStart: 1 / 6,
      playbackRate: 2,
    });
    expect(edit.segments).toEqual([
      { id: 0, sourceEnd: 0.25, sourceStart: 0 },
      {
        id: 2,
        playbackRate: 2,
        sourceEnd: 0.5,
        sourceStart: 0.25,
      },
      {
        id: 1,
        playbackRate: 2,
        sourceEnd: 0.75,
        sourceStart: 0.5,
      },
      {
        id: 3,
        playbackRate: 0.5,
        sourceEnd: 1,
        sourceStart: 0.75,
      },
    ]);
    expect(recordingTimelineRetainedDuration(edit)).toBe(1);
  });

  it("is a no-op for invalid ranges, rates, or already matching content", () => {
    const initial = createRecordingTimelineEdit(42);
    expect(
      setRecordingTimelineRangePlaybackRate(initial, {
        outputEnd: 1,
        outputStart: 0,
        playbackRate: 1,
      }),
    ).toBe(initial);
    expect(
      setRecordingTimelineRangePlaybackRate(initial, {
        outputEnd: 0.5,
        outputStart: 0.5,
        playbackRate: 2,
      }),
    ).toBe(initial);
    expect(
      setRecordingTimelineRangePlaybackRate(initial, {
        outputEnd: 1,
        outputStart: 0,
        playbackRate: Number.NaN,
      }),
    ).toBe(initial);
    expect(recordingTimelineRangePlaybackRate(initial, 0.2, 0.8)).toBe(1);
  });

  it("inherits segment-keyed keyboard settings across every split piece", () => {
    const initial = {
      ...createRecordingTimelineEdit(42),
      deletedKeyboardShortcutFragments: [{ segmentId: 0, shortcutId: 8 }],
      keyboardShortcutPositions: [
        {
          centerX: 0.3,
          centerY: 0.7,
          segmentId: 0,
          shortcutId: 8,
          sizePercent: 125,
        },
      ],
    };
    const edit = setRecordingTimelineRangePlaybackRate(initial, {
      outputEnd: 0.75,
      outputStart: 0.25,
      playbackRate: 2,
    });

    expect(recordingKeyboardShortcutPositions(edit)).toEqual(
      [0, 1, 2].map((segmentId) => ({
        centerX: 0.3,
        centerY: 0.7,
        segmentId,
        shortcutId: 8,
        sizePercent: 125,
      })),
    );
    expect(deletedRecordingKeyboardShortcutFragments(edit)).toEqual(
      [0, 1, 2].map((segmentId) => ({ segmentId, shortcutId: 8 })),
    );
  });
});
