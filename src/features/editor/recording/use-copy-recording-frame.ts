// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useRef, useState } from "react";

import { copyRecordingPreviewFrameToClipboard } from "../api";
import { recordingTimelinePlaybackRanges } from "../timeline/recording-timeline-playback";

import { ResolvedScrubPreviewProps } from "./preview/recording-preview-props";

type Payload = Parameters<typeof copyRecordingPreviewFrameToClipboard>[0];

/** Copies the frame under the playhead, composed as the preview draws it,
 * with the annotations timed on the timeline the edit lays out. */
export function useCopyRecordingFrame(
  {
    artifactId,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    durationMs,
    keyboardEffects,
    recordingTimelineEdit,
  }: ResolvedScrubPreviewProps,
  {
    annotationClips,
    getPositionMs,
    recordingOutput,
  }: Pick<Payload, "annotationClips" | "recordingOutput"> & {
    getPositionMs: () => number;
  },
) {
  const [copyError, setCopyError] = useState<string | null>(null);
  // The copied frame must use the output on screen, which during a resize is
  // the draft; a ref keeps the handler stable without staling the payload.
  const payload = {
    annotationClips,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    keyboardEffects,
    playbackRanges: recordingTimelinePlaybackRanges(
      recordingTimelineEdit,
      durationMs,
    ),
    recordingOutput,
  };
  const copyPayloadRef = useRef(payload);
  copyPayloadRef.current = payload;
  const copyCurrentFrame = useCallback(() => {
    setCopyError(null);
    return copyRecordingPreviewFrameToClipboard({
      ...copyPayloadRef.current,
      artifactId,
      positionMs: getPositionMs(),
    }).catch((cause: unknown) => {
      setCopyError(cause instanceof Error ? cause.message : String(cause));
      // Rethrown so the copy button knows the press failed and skips its check.
      throw cause;
    });
  }, [artifactId, getPositionMs]);

  return { copyCurrentFrame, copyError };
}
