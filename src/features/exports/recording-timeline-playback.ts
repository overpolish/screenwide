// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "./recording-timeline-edit";

export type RecordingTimelinePlaybackRange = {
  sourceEndMs: number;
  sourceStartMs: number;
};

/**
 * Converts editor segments into the source ranges the native player needs.
 * Adjacent cuts remain one playback range; only deleted gaps cause a jump.
 */
export function recordingTimelinePlaybackRanges(
  edit: RecordingTimelineEdit | null | undefined,
  sourceDurationMs: number,
): RecordingTimelinePlaybackRange[] {
  if (!edit || sourceDurationMs <= 0) {
    return [{ sourceEndMs: sourceDurationMs, sourceStartMs: 0 }];
  }
  const ranges: RecordingTimelinePlaybackRange[] = [];
  for (const segment of edit.segments) {
    const sourceStartMs = segment.sourceStart * sourceDurationMs;
    const sourceEndMs = segment.sourceEnd * sourceDurationMs;
    if (
      ranges.length > 0 &&
      Math.abs(ranges[ranges.length - 1].sourceEndMs - sourceStartMs) < 0.5
    ) {
      ranges[ranges.length - 1].sourceEndMs = sourceEndMs;
    } else {
      ranges.push({ sourceEndMs, sourceStartMs });
    }
  }
  return ranges.length > 0
    ? ranges
    : [{ sourceEndMs: sourceDurationMs, sourceStartMs: 0 }];
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

export function recordingTimelinePlaybackRangeFrom(
  ranges: RecordingTimelinePlaybackRange[],
  sourcePositionMs: number,
) {
  const containing = recordingTimelinePlaybackRangeAt(ranges, sourcePositionMs);
  if (containing >= 0) return { index: containing, ranges };
  const next = ranges.findIndex(
    (range) => range.sourceStartMs >= sourcePositionMs,
  );
  return { index: next >= 0 ? next : 0, ranges };
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
