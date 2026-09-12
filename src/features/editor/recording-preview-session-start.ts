// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What a freshly started preview session is told before the frontend starts
// driving it: the surface state React was already holding, and any settings
// that drifted while the session was starting.

import {
  selectRecordingPreviewAudio,
  setRecordingPreviewAudioVolumes,
  setRecordingPreviewComposition,
  setRecordingPreviewCursorEffects,
  setRecordingPreviewEditorSuspended,
  setRecordingPreviewKeyboardEffects,
  setRecordingPreviewZoom,
} from "./api";
import { PreviewZoomRequest } from "./preview-zoom-state";
import { setRecordingPreviewDeletedKeyboardShortcuts } from "./recording-keyboard-timeline-api";

/**
 * Native comes up interactive at 100%, so a session that (re)starts while
 * React owns the workarea has to be told about the suspension, and one that
 * starts with the editor to itself has to be told the zoom. The two are
 * exclusive: React never drives the transform while suspended.
 */
export const pushRecordingPreviewSessionState = ({
  isEditorSuspended,
  nativeEditorOwnsLayout,
  onError,
  sessionId,
  zoomRequest,
}: {
  isEditorSuspended: boolean;
  nativeEditorOwnsLayout: boolean;
  onError: (cause: unknown) => void;
  sessionId: number;
  zoomRequest: PreviewZoomRequest | undefined;
}) => {
  if (isEditorSuspended)
    void setRecordingPreviewEditorSuspended(sessionId, true).catch(onError);
  else if (nativeEditorOwnsLayout && zoomRequest !== undefined)
    void setRecordingPreviewZoom(sessionId, zoomRequest.percent).catch(onError);
};

/**
 * Replays every settings command against the new session. The caller compares
 * settings keys first: this only runs when the user changed something while
 * the session was still starting, so the new worker never keeps stale state.
 */
export const resendRecordingPreviewSettings = ({
  audioTrackVolumes,
  bakeCamera,
  cameraOverlay,
  cursorEffects,
  enabledStreamIndices,
  keyboardDeletions,
  keyboardEffects,
  recordingOutput,
  sessionId,
}: {
  audioTrackVolumes: Parameters<typeof setRecordingPreviewAudioVolumes>[0];
  bakeCamera: boolean;
  cameraOverlay: Parameters<
    typeof setRecordingPreviewComposition
  >[0]["cameraOverlay"];
  cursorEffects: Parameters<typeof setRecordingPreviewCursorEffects>[0];
  enabledStreamIndices: Parameters<typeof selectRecordingPreviewAudio>[0];
  keyboardDeletions: Parameters<
    typeof setRecordingPreviewDeletedKeyboardShortcuts
  >[0];
  keyboardEffects: Parameters<typeof setRecordingPreviewKeyboardEffects>[0];
  recordingOutput: Parameters<
    typeof setRecordingPreviewComposition
  >[0]["recordingOutput"];
  sessionId: number;
}) =>
  Promise.all([
    selectRecordingPreviewAudio(enabledStreamIndices, sessionId),
    setRecordingPreviewAudioVolumes(audioTrackVolumes, sessionId),
    setRecordingPreviewCursorEffects(cursorEffects, sessionId),
    setRecordingPreviewKeyboardEffects(keyboardEffects, sessionId),
    setRecordingPreviewDeletedKeyboardShortcuts(keyboardDeletions, sessionId),
    setRecordingPreviewComposition({
      bakeCamera,
      cameraOverlay,
      recordingOutput,
      sessionId,
    }),
  ]);
