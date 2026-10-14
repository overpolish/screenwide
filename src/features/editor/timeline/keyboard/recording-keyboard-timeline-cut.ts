// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  cutRecordingTimeline,
  RecordingTimelineEdit,
} from "../editing/recording-timeline-edit";

import { deletedRecordingKeyboardShortcutFragments } from "./recording-keyboard-timeline-edit";

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
  const deleted = deletedRecordingKeyboardShortcutFragments(edit)
    .filter((fragment) => fragment.segmentId === sourceSegment.id)
    .map((fragment) => ({ ...fragment, segmentId }));
  if (deleted.length === 0) return next;
  return {
    ...next,
    deletedKeyboardShortcutFragments: [
      ...deletedRecordingKeyboardShortcutFragments(edit),
      ...deleted,
    ].sort((a, b) => a.shortcutId - b.shortcutId || a.segmentId - b.segmentId),
  };
}
