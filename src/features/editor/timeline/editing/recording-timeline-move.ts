// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingTimelineEdit } from "../recording-timeline-types";

/**
 * Moves the segment `segmentId`, screen, camera and audio together, to the
 * boundary `boundary`: 0 before the first segment, the segment count after
 * the last, and `i` between segments `i - 1` and `i`, counted before the
 * move. The segments after it close the gap. A move that changes nothing
 * returns the edit itself, which keeps it out of undo history.
 */
export function moveRecordingTimelineSegment(
  edit: RecordingTimelineEdit,
  { boundary, segmentId }: { boundary: number; segmentId: number },
): RecordingTimelineEdit {
  const from = edit.segments.findIndex((segment) => segment.id === segmentId);
  if (from < 0 || !Number.isInteger(boundary)) return edit;
  const to = Math.max(0, Math.min(edit.segments.length, boundary));
  if (to === from || to === from + 1) return edit;
  const segments = [...edit.segments];
  const [moved] = segments.splice(from, 1);
  segments.splice(to > from ? to - 1 : to, 0, moved);
  return { ...edit, segments };
}

/**
 * Puts every segment back in the order it was recorded, changing nothing
 * else: cuts, speeds and the segments' own bounds stay, and segments that
 * now meet are not joined, since nothing says they belong together. Already
 * in recorded order, it returns the edit itself, which keeps it out of undo
 * history.
 */
export function restoreRecordingTimelineOrder(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  if (!isRecordingTimelineReordered(edit)) return edit;
  return {
    ...edit,
    segments: [...edit.segments].sort((a, b) => a.sourceStart - b.sourceStart),
  };
}

/** Whether any segment plays before one recorded earlier. */
export const isRecordingTimelineReordered = (edit: RecordingTimelineEdit) =>
  edit.segments.some(
    (segment, index) =>
      index > 0 && segment.sourceStart < edit.segments[index - 1].sourceStart,
  );

/** The boundary that swaps the segment at `index` with its neighbour in
 * `direction`, or null at that end of the timeline. */
export function neighbourBoundary(
  segmentCount: number,
  index: number,
  direction: -1 | 1,
): number | null {
  if (index < 0) return null;
  if (direction < 0) return index > 0 ? index - 1 : null;
  return index + 1 < segmentCount ? index + 2 : null;
}
