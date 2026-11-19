// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useEffect, useRef, useState } from "react";
import { create } from "zustand";

import { copyRecordingPreviewFrameToClipboard } from "../api";
import {
  playedMs,
  recordingStripTime,
} from "../timeline/recording-strip/recording-strip-time";
import { recordingTimelinePlaybackRanges } from "../timeline/recording-timeline-playback";

import { ResolvedScrubPreviewProps } from "./preview/recording-preview-props";

type Payload = Parameters<typeof copyRecordingPreviewFrameToClipboard>[0];

/** How to copy the frame under the open recording's playhead, for the
 * title bar's Copy Frame, which stands outside the preview that owns the
 * player; null while no recording preview is mounted. */
export const useRecordingFrameCopy = create<{
  copy: (() => Promise<unknown>) | null;
}>(() => ({ copy: null }));

/** Where the open recording's playhead stands, in the recording and in what
 * plays, in ms: what a cue made in the editor shows. Null while no
 * recording preview is mounted. */
export const usePlayheadMoment = create<{
  moment: (() => { playedMs: number; sourceMs: number }) | null;
}>(() => ({ moment: null }));

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
  useEffect(() => {
    const ownEdit =
      recordingTimelineEdit?.artifactId === artifactId
        ? recordingTimelineEdit
        : null;
    const map = ownEdit
      ? recordingStripTime(ownEdit, durationMs / 1000).map
      : null;
    usePlayheadMoment.setState({
      moment: () => {
        const sourceMs = getPositionMs();
        return {
          playedMs: map ? playedMs(map, sourceMs / 1000) : sourceMs,
          sourceMs,
        };
      },
    });
    return () => {
      usePlayheadMoment.setState({ moment: null });
    };
  }, [artifactId, durationMs, getPositionMs, recordingTimelineEdit]);

  return { copyError };
}
