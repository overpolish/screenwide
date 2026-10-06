// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { recordingTimelineOutputToSource } from "../timeline/editing/recording-timeline-edit";

import { ResolvedScrubPreviewProps } from "./preview/recording-preview-props";
import {
  RecordingFrameRequest,
  saveRecordingProjectStill,
} from "./recording-frame-api";

/** Long enough that a drag or a run of nudges composes one still, not many. */
const SETTLE_MS = 1_000;

/**
 * Keeps the project browser's still of this recording in step with its edit:
 * composed once the editor opens it and again once each change settles, at
 * the middle of the edited timeline. It follows the committed canvas, not a
 * resize in progress.
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
    const middle =
      edit?.artifactId === artifactId
        ? recordingTimelineOutputToSource(edit, 0.5)
        : 0.5;
    const timer = window.setTimeout(() => {
      // A still is a convenience: one that cannot be made now is made at the
      // next change, and the browser shows a movie frame meanwhile.
      saveRecordingProjectStill({
        annotationClips,
        artifactId,
        bakeCamera,
        cameraOverlay,
        cursorEffects,
        keyboardEffects,
        positionMs: middle * durationMs,
        recordingOutput,
      }).catch(() => undefined);
    }, SETTLE_MS);
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
