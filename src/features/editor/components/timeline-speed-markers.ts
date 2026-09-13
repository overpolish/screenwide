// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
} from "../recording-timeline-edit";

/** The clips that play at another rate, in output order. */
export const speedMarkedSegments = (edit: RecordingTimelineEdit) =>
  layoutRecordingTimelineSegments(edit).filter(
    (segment) => (segment.playbackRate ?? 1) !== 1,
  );
