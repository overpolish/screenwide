// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  deletedRecordingKeyboardShortcutFragments,
  recordingKeyboardShortcutPositions,
} from "./recording-keyboard-timeline-edit";
import {
  cutRecordingTimeline,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";

export function cutKeyboardTimeline(
  edit: RecordingTimelineEdit,
  sourcePosition: number,
): RecordingTimelineEdit {
  const sourceSegment = edit.segments.find(
    (segment) =>
      sourcePosition > segment.sourceStart &&
      sourcePosition < segment.sourceEnd,
  );
  const next = cutRecordingTimeline(edit, sourcePosition);
  if (!sourceSegment || next === edit) return next;
  const segmentId = edit.nextSegmentId;
  const positions = recordingKeyboardShortcutPositions(edit)
    .filter((position) => position.segmentId === sourceSegment.id)
    .map((position) => ({ ...position, segmentId }));
  const deleted = deletedRecordingKeyboardShortcutFragments(edit)
    .filter((fragment) => fragment.segmentId === sourceSegment.id)
    .map((fragment) => ({ ...fragment, segmentId }));
  return {
    ...next,
    ...(positions.length > 0
      ? {
          keyboardShortcutPositions: [
            ...recordingKeyboardShortcutPositions(edit),
            ...positions,
          ].sort(
            (a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId,
          ),
        }
      : {}),
    ...(deleted.length > 0
      ? {
          deletedKeyboardShortcutFragments: [
            ...deletedRecordingKeyboardShortcutFragments(edit),
            ...deleted,
          ].sort(
            (a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId,
          ),
        }
      : {}),
  };
}
