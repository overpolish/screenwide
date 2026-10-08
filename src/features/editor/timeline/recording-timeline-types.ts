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
