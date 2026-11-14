// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingTimelineEdit } from "../../../bindings/RecordingTimelineEdit";
import type { RecordingTimelineSegment } from "../../../bindings/RecordingTimelineSegment";

export type { RecordingTimelineEdit, RecordingTimelineSegment };

export type RecordingTimelineTrimEdge = "end" | "start";

export type RecordingTimelineLayoutSegment = RecordingTimelineSegment & {
  /** Normalized position in the retained, magnetic output timeline. */
  outputEnd: number;
  /** Normalized position in the retained, magnetic output timeline. */
  outputStart: number;
};

/** Whether `segment` is a gap, time inserted with nothing recorded behind
 * it, rather than a stretch of the recording: a gap lies past the
 * recording's end (`recording-strip-gaps.ts`). */
export const isGapSegment = (segment: { sourceStart: number }) =>
  segment.sourceStart >= 1;
