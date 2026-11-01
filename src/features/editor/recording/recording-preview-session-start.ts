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
} from "../api";
import { RecordingOutputSettings } from "../screenshot/screenshot-output";
import {
  RecordingPreviewKeyboardDeletions,
  setRecordingPreviewDeletedKeyboardShortcuts,
} from "../timeline/keyboard/recording-keyboard-timeline-api";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "../types";

/** The settings a session runs from, as the frontend holds them at one instant. */
export type RecordingPreviewSessionSettings = {
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  enabledStreamIndices: number[];
  keyboardDeletions: RecordingPreviewKeyboardDeletions;
  keyboardEffects: KeyboardEffectSettings;
  recordingOutput: RecordingOutputSettings;
};

export const recordingPreviewSessionSettingsKey = (
  settings: RecordingPreviewSessionSettings,
) => JSON.stringify(settings);

/**
 * Native comes up interactive, so a session that (re)starts while React owns
 * the workarea has to be told about the suspension.
 */
export const pushRecordingPreviewSessionState = ({
  isEditorSuspended,
  onError,
  sessionId,
}: {
  isEditorSuspended: boolean;
  onError: (cause: unknown) => void;
  sessionId: number;
}) => {
  if (isEditorSuspended)
    void setRecordingPreviewEditorSuspended(sessionId, true).catch(onError);
};

/**
 * Settles a session that started while its settings moved on. The session was
 * started from the settings behind `initialKey`; if the frontend now holds
 * different ones, every settings command is replayed against it, so the new
 * worker never keeps stale state.
 */
export const settleRecordingPreviewSettings = ({
  initialKey,
  onError,
  sessionId,
  settings,
}: {
  initialKey: string;
  onError: (cause: unknown) => void;
  sessionId: number;
  settings: RecordingPreviewSessionSettings;
}) => {
  if (recordingPreviewSessionSettingsKey(settings) === initialKey) return;
  const {
    audioTrackVolumes,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    enabledStreamIndices,
    keyboardDeletions,
    keyboardEffects,
    recordingOutput,
  } = settings;
  void Promise.all([
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
  ]).catch(onError);
};
