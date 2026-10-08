// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { recordingTimelineOutputToSource } from "../timeline/editing/recording-timeline-edit";

import { ResolvedScrubPreviewProps } from "./preview/recording-preview-props";
import {
  RecordingFrameRequest,
  requestRecordingProjectPictures,
} from "./recording-frame-api";

/** Long enough that a drag or a run of nudges sends one edit, not dozens.
 * The app waits a little more for the edit to rest before composing. */
const REPORT_MS = 250;
/** Enough to follow a take across a card in the browser. */
const STRIP_FRAMES = 12;

/**
 * Keeps the project browser's pictures of this recording in step with its
 * edit: the still, at the middle of the edited timeline, and the scrub
 * strip, at the middle of each twelfth of it. The edit is handed to the app
 * when the editor opens the recording and again after each change, and the
 * app composes both in one pass once it rests, or before letting the
 * recording go if the window closes first. They follow the committed
 * canvas, not a resize in progress.
 */
export function useProjectStill(
  {
    artifactId,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    durationMs,
    keyboardEffects,
    recordingOutput,
    recordingTimelineEdit: edit,
  }: ResolvedScrubPreviewProps,
  {
    annotationClips,
    hasPicture,
  }: Pick<RecordingFrameRequest, "annotationClips"> & {
    /** Whether the screen is in the preview to compose. */
    hasPicture: boolean;
  },
) {
  useEffect(() => {
    if (!hasPicture || !recordingOutput || durationMs <= 0) return;
    // A point along the edited timeline, from 0 to 1, as a source position.
    const sourceMs = (output: number) =>
      (edit?.artifactId === artifactId
        ? recordingTimelineOutputToSource(edit, output)
        : output) * durationMs;
    const request: RecordingFrameRequest = {
      annotationClips,
      artifactId,
      bakeCamera,
      cameraOverlay,
      cursorEffects,
      keyboardEffects,
      positionMs: 0,
      recordingOutput,
    };
    const timer = window.setTimeout(() => {
      // A convenience: pictures that cannot be made now are made at the next
      // change, and the browser shows a movie frame, and no scrubbing,
      // meanwhile.
      requestRecordingProjectPictures({
        request,
        stillPositionMs: sourceMs(0.5),
        stripPositionsMs: Array.from({ length: STRIP_FRAMES }, (_, index) =>
          sourceMs((index + 0.5) / STRIP_FRAMES),
        ),
      }).catch(() => undefined);
    }, REPORT_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [
    annotationClips,
    artifactId,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    durationMs,
    edit,
    hasPicture,
    keyboardEffects,
    recordingOutput,
  ]);
}
