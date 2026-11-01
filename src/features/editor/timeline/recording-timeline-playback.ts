// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "./editing/recording-timeline-edit";

export type RecordingTimelinePlaybackRange = {
  playbackRate: number;
  sourceEndMs: number;
  sourceStartMs: number;
};

/**
 * Converts editor segments into the source ranges the native player needs.
 * Adjacent cuts remain one playback range; only deleted gaps cause a jump.
 * Ranges are whole milliseconds, as the native side counts them: a segment
 * edge is a share of the recording, which multiplied back out lands a hair
 * off the millisecond it was made at.
 */
export function recordingTimelinePlaybackRanges(
  edit: RecordingTimelineEdit | null | undefined,
  sourceDurationMs: number,
): RecordingTimelinePlaybackRange[] {
  if (!edit || sourceDurationMs <= 0) {
    return [
      { playbackRate: 1, sourceEndMs: sourceDurationMs, sourceStartMs: 0 },
    ];
  }
  const ranges: RecordingTimelinePlaybackRange[] = [];
  for (const segment of edit.segments) {
    const sourceStartMs = Math.round(segment.sourceStart * sourceDurationMs);
    const sourceEndMs = Math.round(segment.sourceEnd * sourceDurationMs);
    const playbackRate = segment.playbackRate ?? 1;
    if (
      ranges.length > 0 &&
      Math.abs(ranges[ranges.length - 1].sourceEndMs - sourceStartMs) < 0.5 &&
      ranges[ranges.length - 1].playbackRate === playbackRate
    ) {
      ranges[ranges.length - 1].sourceEndMs = sourceEndMs;
    } else {
      ranges.push({ playbackRate, sourceEndMs, sourceStartMs });
    }
  }
  return ranges.length > 0
    ? ranges
    : [{ playbackRate: 1, sourceEndMs: sourceDurationMs, sourceStartMs: 0 }];
}

export function recordingTimelinePlaybackRangeAt(
  ranges: RecordingTimelinePlaybackRange[],
  sourcePositionMs: number,
) {
  return ranges.findIndex(
    (range) =>
      sourcePositionMs >= range.sourceStartMs &&
      sourcePositionMs < range.sourceEndMs,
  );
}

/**
 * Where playback from a playhead at `sourcePositionMs` goes on from. At the
 * recording's end the playhead shows its last decodable millisecond, the
 * still the native player can draw there, which leaves nothing to play: it
 * counts as the end itself, past which playback starts over.
 */
export function recordingTimelinePlaybackPosition(
  sourcePositionMs: number,
  sourceDurationMs: number,
) {
  return sourceDurationMs > 0 && sourcePositionMs >= sourceDurationMs - 1
    ? sourceDurationMs
    : sourcePositionMs;
}

/**
 * The range playback from `sourcePositionMs` starts in: the one holding it,
 * or, for a moment cut away, the one played after its source predecessor,
 * where the cut leaves the playhead. Past the last range it starts over.
 */
export function recordingTimelinePlaybackRangeFrom(
  ranges: RecordingTimelinePlaybackRange[],
  sourcePositionMs: number,
) {
  const containing = recordingTimelinePlaybackRangeAt(ranges, sourcePositionMs);
  if (containing >= 0) return { index: containing, ranges };
  let predecessor = -1;
  let earliest = -1;
  ranges.forEach((range, index) => {
    if (
      range.sourceEndMs <= sourcePositionMs &&
      (predecessor < 0 || range.sourceEndMs > ranges[predecessor].sourceEndMs)
    )
      predecessor = index;
    if (earliest < 0 || range.sourceStartMs < ranges[earliest].sourceStartMs)
      earliest = index;
  });
  const index =
    predecessor < 0
      ? earliest
      : predecessor + 1 < ranges.length
        ? predecessor + 1
        : 0;
  return { index: Math.max(0, index), ranges };
}

export function recordingTimelinePlaybackRangesFrom(
  ranges: RecordingTimelinePlaybackRange[],
  sourcePositionMs: number,
) {
  const { index } = recordingTimelinePlaybackRangeFrom(
    ranges,
    sourcePositionMs,
  );
  return ranges.slice(index).map((range, rangeIndex) => ({
    playbackRate: range.playbackRate,
    sourceEndMs: Math.round(range.sourceEndMs),
    sourceStartMs: Math.round(
      rangeIndex === 0
        ? Math.max(sourcePositionMs, range.sourceStartMs)
        : range.sourceStartMs,
    ),
  }));
}

export function recordingTimelinePlaybackDurationMs(
  edit: RecordingTimelineEdit | null | undefined,
  sourceDurationMs: number,
) {
  return edit
    ? recordingTimelineRetainedDuration(edit) * sourceDurationMs
    : sourceDurationMs;
}
