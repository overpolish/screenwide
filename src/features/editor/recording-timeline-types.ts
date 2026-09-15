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
  /** Annotation timing remains in source milliseconds across cuts and speed changes. */
  annotationClips?: RecordingAnnotationClip[];
};

export type RecordingTimelineTrimEdge = "end" | "start";

export type RecordingTimelineLayoutSegment = RecordingTimelineSegment & {
  /** Normalized position in the retained, magnetic output timeline. */
  outputEnd: number;
  /** Normalized position in the retained, magnetic output timeline. */
  outputStart: number;
};
