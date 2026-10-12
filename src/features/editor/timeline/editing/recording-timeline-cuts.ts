// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  deletedRecordingKeyboardShortcutFragments,
  recordingKeyboardShortcutPositions,
} from "../keyboard/recording-keyboard-timeline-edit";

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";
import { freeSourceBounds } from "./recording-timeline-source-order";

const CUT_EPSILON = 1e-9;

/**
 * A stretch of the recording left out, shown where restoring it would put
 * it back: after the segment that ends where it starts in the source, or at
 * the start of the segment earliest in the source when none does.
 */
export type RecordingTimelineCut = {
  /** The segment that starts where the cut ends in the source, or none at
   * the end of the recording. */
  followingSegmentId: number | null;
  /** Where it is, in normalized output time: the end of the segment it
   * follows, or the start of the one it comes before. */
  outputPosition: number;
  /** The segment the cut follows in the source, or none at the start. */
  segmentId: number | null;
  /** What was left out, in normalized source time. */
  sourceEnd: number;
  sourceStart: number;
};

/**
 * Every stretch of source left out, in output order. A blade split with
 * nothing removed, or a jump a reordered timeline makes, leaves nothing to
 * bring back and is no cut.
 */
export const recordingTimelineCuts = (
  edit: RecordingTimelineEdit,
): RecordingTimelineCut[] => {
  const layout = layoutRecordingTimelineSegments(edit);
  if (layout.length === 0) return [];
  const bySource = [...layout].sort((a, b) => a.sourceStart - b.sourceStart);
  const cuts: RecordingTimelineCut[] = [];
  bySource.forEach((segment, index) => {
    const previous = index > 0 ? bySource[index - 1] : null;
    const gapStart = previous?.sourceEnd ?? 0;
    if (segment.sourceStart - gapStart > CUT_EPSILON)
      cuts.push({
        followingSegmentId: segment.id,
        outputPosition: previous?.outputEnd ?? segment.outputStart,
        segmentId: previous?.id ?? null,
        sourceEnd: segment.sourceStart,
        sourceStart: gapStart,
      });
  });
  const last = bySource[bySource.length - 1];
  if (1 - last.sourceEnd > CUT_EPSILON)
    cuts.push({
      followingSegmentId: null,
      outputPosition: last.outputEnd,
      segmentId: last.id,
      sourceEnd: 1,
      sourceStart: last.sourceEnd,
    });
  return cuts.sort((a, b) => a.outputPosition - b.outputPosition);
};

/** Shortcut moves and deletions are kept per segment, so a segment carrying
 * any cannot be folded into its neighbour without changing what they cover. */
export const hasKeyboardSegmentEdits = (
  edit: RecordingTimelineEdit,
  segmentId: number,
) =>
  recordingKeyboardShortcutPositions(edit).some(
    (position) => position.segmentId === segmentId,
  ) ||
  deletedRecordingKeyboardShortcutFragments(edit).some(
    (fragment) => fragment.segmentId === segmentId,
  );

/**
 * Puts back `cut`, as dragging the edge of the segment before it in the
 * source out over it would. Where the segment after it in the source plays
 * straight after, at the same rate, and neither carries a shortcut edit,
 * the two become one segment again; otherwise the restored stretch plays at
 * the rate of the segment before it. At the start of the recording the
 * segment earliest in the source reaches back to it, and at the end the
 * last one reaches on. A cut that is no longer there returns the edit
 * unchanged, which keeps it out of undo history.
 */
export function restoreRecordingTimelineCut(
  edit: RecordingTimelineEdit,
  cut: Pick<RecordingTimelineCut, "followingSegmentId" | "segmentId">,
): RecordingTimelineEdit {
  const index = (id: number | null) =>
    id === null ? -1 : edit.segments.findIndex((segment) => segment.id === id);
  const before = index(cut.segmentId);
  const after = index(cut.followingSegmentId);
  const segments = [...edit.segments];
  if (cut.segmentId === null) {
    if (after < 0 || segments[after].sourceStart <= CUT_EPSILON) return edit;
    const earlier = segments.some(
      (segment) => segment.sourceStart < segments[after].sourceStart,
    );
    if (earlier) return edit;
    segments[after] = { ...segments[after], sourceStart: 0 };
    return { ...edit, segments };
  }
  if (before < 0) return edit;
  const segment = segments[before];
  const upper = freeSourceBounds(segments, before).upper;
  if (upper - segment.sourceEnd <= CUT_EPSILON) return edit;
  if (cut.followingSegmentId === null) {
    if (upper < 1) return edit;
    segments[before] = { ...segment, sourceEnd: 1 };
    return { ...edit, segments };
  }
  if (after < 0 || Math.abs(segments[after].sourceStart - upper) > CUT_EPSILON)
    return edit;
  const next = segments[after];
  const rejoins =
    after === before + 1 &&
    (segment.playbackRate ?? 1) === (next.playbackRate ?? 1) &&
    !hasKeyboardSegmentEdits(edit, segment.id) &&
    !hasKeyboardSegmentEdits(edit, next.id);
  if (rejoins) {
    segments.splice(before, 2, { ...segment, sourceEnd: next.sourceEnd });
  } else {
    segments[before] = { ...segment, sourceEnd: next.sourceStart };
  }
  return { ...edit, segments };
}
