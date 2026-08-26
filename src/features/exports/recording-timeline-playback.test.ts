// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingTimelineEdit } from "./recording-timeline-edit";
import {
  recordingTimelinePlaybackDurationMs,
  recordingTimelinePlaybackRangeAt,
  recordingTimelinePlaybackRanges,
  recordingTimelinePlaybackRangesFrom,
} from "./recording-timeline-playback";

const edit: RecordingTimelineEdit = {
  artifactId: 1,
  nextSegmentId: 4,
  segments: [
    { id: 0, sourceEnd: 0.25, sourceStart: 0 },
    { id: 1, sourceEnd: 0.5, sourceStart: 0.25 },
    { id: 3, sourceEnd: 1, sourceStart: 0.75 },
  ],
};

describe("recording timeline playback", () => {
  it("coalesces cuts but preserves deleted gaps", () => {
    expect(recordingTimelinePlaybackRanges(edit, 8_000)).toEqual([
      { sourceEndMs: 4_000, sourceStartMs: 0 },
      { sourceEndMs: 8_000, sourceStartMs: 6_000 },
    ]);
  });

  it("finds retained ranges without treating their end as playable", () => {
    const ranges = recordingTimelinePlaybackRanges(edit, 8_000);
    expect(recordingTimelinePlaybackRangeAt(ranges, 3_999)).toBe(0);
    expect(recordingTimelinePlaybackRangeAt(ranges, 4_000)).toBe(-1);
    expect(recordingTimelinePlaybackRangeAt(ranges, 6_000)).toBe(1);
  });

  it("reports magnetic duration", () => {
    expect(recordingTimelinePlaybackDurationMs(edit, 8_000)).toBe(6_000);
  });

  it("starts native playback inside the current retained range", () => {
    const ranges = recordingTimelinePlaybackRanges(edit, 8_000);
    expect(recordingTimelinePlaybackRangesFrom(ranges, 2_500)).toEqual([
      { sourceEndMs: 4_000, sourceStartMs: 2_500 },
      { sourceEndMs: 8_000, sourceStartMs: 6_000 },
    ]);
    expect(recordingTimelinePlaybackRangesFrom(ranges, 5_000)).toEqual([
      { sourceEndMs: 8_000, sourceStartMs: 6_000 },
    ]);
  });
});
