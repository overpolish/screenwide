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

const CUT_EPSILON = 1e-9;

/**
 * A stretch of the recording left out: between two neighbouring segments,
 * or before the first or after the last.
 */
export type RecordingTimelineCut = {
  /** The segment the cut comes before, or none at the end. */
  followingSegmentId: number | null;
  /** Where it is, in normalized output time: the join, or an end. */
  outputPosition: number;
  /** The segment the cut follows, or none at the start. */
  segmentId: number | null;
  /** What was left out, in normalized source time. */
  sourceEnd: number;
  sourceStart: number;
};

/**
 * Every place source was left out, in output order: the start of the
 * recording, each join, and the end. A blade split with nothing removed is a
 * join too, but there is nothing to bring back at it.
 */
export const recordingTimelineCuts = (
  edit: RecordingTimelineEdit,
): RecordingTimelineCut[] => {
  const layout = layoutRecordingTimelineSegments(edit);
  if (layout.length === 0) return [];
  const first = layout[0];
  const last = layout[layout.length - 1];
  const joins = layout.flatMap((segment, index) => {
    const next = index + 1 < layout.length ? layout[index + 1] : null;
    return next && next.sourceStart - segment.sourceEnd > CUT_EPSILON
      ? [
          {
            followingSegmentId: next.id,
            outputPosition: segment.outputEnd,
            segmentId: segment.id,
            sourceEnd: next.sourceStart,
            sourceStart: segment.sourceEnd,
          },
        ]
      : [];
  });
  return [
    ...(first.sourceStart > CUT_EPSILON
      ? [
          {
            followingSegmentId: first.id,
            outputPosition: 0,
            segmentId: null,
            sourceEnd: first.sourceStart,
            sourceStart: 0,
          },
        ]
      : []),
    ...joins,
    ...(1 - last.sourceEnd > CUT_EPSILON
      ? [
          {
            followingSegmentId: null,
            outputPosition: 1,
            segmentId: last.id,
            sourceEnd: 1,
            sourceStart: last.sourceEnd,
          },
        ]
      : []),
  ];
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
 * Puts back `cut`, as dragging the neighbouring edge out over it would. At a
 * join, where the two segments play at the same rate and neither carries a
 * shortcut edit, they become one segment again, so the join goes along with
 * the gap; otherwise the restored stretch plays at the rate of the segment
 * before it. At the start or end of the recording the segment beside it
 * reaches back to that end. A cut that is no longer there returns the edit
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
    if (after !== 0 || segments[0].sourceStart <= CUT_EPSILON) return edit;
    segments[0] = { ...segments[0], sourceStart: 0 };
    return { ...edit, segments };
  }
  if (cut.followingSegmentId === null) {
    const lastIndex = segments.length - 1;
    if (
      before !== lastIndex ||
      1 - segments[lastIndex].sourceEnd <= CUT_EPSILON
    )
      return edit;
    segments[lastIndex] = { ...segments[lastIndex], sourceEnd: 1 };
    return { ...edit, segments };
  }
  if (before < 0 || after !== before + 1) return edit;
  const segment = segments[before];
  const next = segments[after];
  if (next.sourceStart - segment.sourceEnd <= CUT_EPSILON) return edit;
  const rejoins =
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
