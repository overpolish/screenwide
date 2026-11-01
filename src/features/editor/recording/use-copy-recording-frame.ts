// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useEffect, useRef, useState } from "react";
import { create } from "zustand";

import { copyRecordingPreviewFrameToClipboard } from "../api";
import { recordingTimelinePlaybackRanges } from "../timeline/recording-timeline-playback";

import { ResolvedScrubPreviewProps } from "./preview/recording-preview-props";

type Payload = Parameters<typeof copyRecordingPreviewFrameToClipboard>[0];

/** How to copy the frame under the open recording's playhead, for the
 * title bar's Copy Frame, which stands outside the preview that owns the
 * player; null while no recording preview is mounted. */
export const useRecordingFrameCopy = create<{
  copy: (() => Promise<unknown>) | null;
}>(() => ({ copy: null }));

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
      // Said under the picture, where the frame was copied from.
      setCopyError(cause instanceof Error ? cause.message : String(cause));
    });
  }, [artifactId, getPositionMs]);
  useEffect(() => {
    useRecordingFrameCopy.setState({ copy: copyCurrentFrame });
    return () => {
      useRecordingFrameCopy.setState({ copy: null });
    };
  }, [copyCurrentFrame]);

  return { copyError };
}
