// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingAnnotationClip } from "./recording-annotations";

export type RecordingTimelineSegment = {
  id: number;
  /** Normalized position in the original recording, before magnetic edits. */
  sourceEnd: number;
  /** Normalized position in the original recording, before magnetic edits. */
  sourceStart: number;
  /** Playback rate for this retained source range. */
  playbackRate?: number;
};

export type RecordingTimelineEdit = {
  artifactId: number;
  nextSegmentId: number;
  segments: RecordingTimelineSegment[];
  /** Annotation clips stay in source milliseconds across cuts and speed
   * changes; their arrivals and leavings play in output time over what the
   * timeline keeps of each clip. */
  annotationClips?: RecordingAnnotationClip[];
};

export type RecordingTimelineTrimEdge = "end" | "start";

export type RecordingTimelineLayoutSegment = RecordingTimelineSegment & {
  /** Normalized position in the retained, magnetic output timeline. */
  outputEnd: number;
  /** Normalized position in the retained, magnetic output timeline. */
  outputStart: number;
};
