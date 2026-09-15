// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useRef, useState } from "react";

import { copyRecordingPreviewFrameToClipboard } from "./api";
type Payload = Parameters<typeof copyRecordingPreviewFrameToClipboard>[0];
export function useCopyRecordingFrame({
  annotationClips,
  artifactId,
  bakeCamera,
  cameraOverlay,
  cursorEffects,
  getPositionMs,
  keyboardEffects,
  recordingOutput: effectiveRecordingOutput,
}: Omit<Payload, "positionMs"> & { getPositionMs: () => number }) {
  const [copyError, setCopyError] = useState<string | null>(null);
  // The copied frame must use the output on screen, which during a resize is
  // the draft; a ref keeps the handler stable without staling the payload.
  const copyPayloadRef = useRef({
    annotationClips,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    keyboardEffects,
    recordingOutput: effectiveRecordingOutput,
  });
  copyPayloadRef.current = {
    annotationClips,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    keyboardEffects,
    recordingOutput: effectiveRecordingOutput,
  };
  const copyCurrentFrame = useCallback(() => {
    setCopyError(null);
    return copyRecordingPreviewFrameToClipboard({
      annotationClips: copyPayloadRef.current.annotationClips,
      artifactId,
      bakeCamera: copyPayloadRef.current.bakeCamera,
      cameraOverlay: copyPayloadRef.current.cameraOverlay,
      cursorEffects: copyPayloadRef.current.cursorEffects,
      keyboardEffects: copyPayloadRef.current.keyboardEffects,
      positionMs: getPositionMs(),
      recordingOutput: copyPayloadRef.current.recordingOutput,
    }).catch((cause: unknown) => {
      setCopyError(cause instanceof Error ? cause.message : String(cause));
      // Rethrown so the copy button knows the press failed and skips its check.
      throw cause;
    });
  }, [artifactId, getPositionMs]);

  return { copyCurrentFrame, copyError };
}
