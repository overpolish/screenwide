// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { inheritRecordingKeyboardSegmentEdits } from "../keyboard/recording-keyboard-timeline-edit";

import {
  cutSegments,
  mergeAddedSegments,
  removedParts,
  sharesOf,
  stillRemoved,
} from "./recording-timeline-cut-ranges";
import {
  RecordingTimelineEdit,
  RecordingTimelineSegment,
} from "./recording-timeline-edit";

import type { RecordingSilenceCut } from "../../../../bindings/RecordingSilenceCut";
import type { SilenceCut } from "../../../../bindings/SilenceCut";

export type { SilenceCut };

/** The recorded silences, cut down to what is still cut out. */
const stillRemovedSilences = (
  edit: RecordingTimelineEdit,
): RecordingSilenceCut[] => stillRemoved(edit.segments, edit.silenceCuts ?? []);

/**
 * How many of the pauses Remove silences cut are still cut, and how much
 * shorter they make the recording, in milliseconds of the recording at the
 * rate each played at.
 */
export function recordingTimelineSilenceSummary(
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
) {
  const removed = stillRemovedSilences(edit);
  return {
    count: (edit.silenceCuts ?? []).filter(
      (cut) =>
        removedParts(edit.segments, {
          end: cut.sourceEnd,
          start: cut.sourceStart,
        }).length > 0,
    ).length,
    durationMs: removed.reduce(
      (total, cut) =>
        total +
        ((cut.sourceEnd - cut.sourceStart) * sourceDurationMs) /
          cut.playbackRate,
      0,
    ),
  };
}

/**
 * Cuts `cuts`, in milliseconds of a recording `sourceDurationMs` long, out of
 * whatever of them the edit still keeps, and records what it took for Restore
 * all. A cut never takes the whole recording; one that would is left out.
 */
export function removeRecordingTimelineSilences(
  edit: RecordingTimelineEdit,
  cuts: readonly SilenceCut[],
  sourceDurationMs: number,
): RecordingTimelineEdit {
  if (sourceDurationMs <= 0) return edit;
  const ranges = sharesOf(cuts, sourceDurationMs);
  if (ranges.length === 0) return edit;
  const { descendantIds, nextSegmentId, segments, taken } = cutSegments(
    edit,
    ranges,
  );
  if (taken.length === 0 || segments.length === 0) return edit;
  return inheritRecordingKeyboardSegmentEdits(
    edit,
    {
      ...edit,
      nextSegmentId,
      segments,
      silenceCuts: [...stillRemovedSilences(edit), ...taken].sort(
        (a, b) => a.sourceStart - b.sourceStart,
      ),
    },
    descendantIds,
  );
}

/**
 * Brings back every part of the pauses Remove silences cut that is still cut,
 * each at the rate it played at. A restored part that meets a neighbour at
 * the same rate becomes one with it, unless either carries a shortcut edit.
 */
export function restoreRecordingTimelineSilences(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  const removed = stillRemovedSilences(edit);
  if (removed.length === 0) {
    if (!edit.silenceCuts?.length) return edit;
    const { silenceCuts: _, ...rest } = edit;
    return rest;
  }
  let nextSegmentId = edit.nextSegmentId;
  const added = removed.map((cut): RecordingTimelineSegment => ({
    id: nextSegmentId++,
    sourceEnd: cut.sourceEnd,
    sourceStart: cut.sourceStart,
    ...(cut.playbackRate === 1 ? {} : { playbackRate: cut.playbackRate }),
  }));
  const segments = mergeAddedSegments(edit, added);
  const { silenceCuts: _, ...rest } = edit;
  return { ...rest, nextSegmentId, segments };
}
