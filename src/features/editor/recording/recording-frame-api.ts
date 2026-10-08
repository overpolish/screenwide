// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  normalizedCameraOverlay,
  normalizedCursorEffects,
  normalizedKeyboardEffects,
} from "../preview/preview-settings-normalization";
import {
  normalizedScreenshotOutput,
  RecordingOutputSettings,
} from "../screenshot/screenshot-output";
import {
  CameraOverlaySettings,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "../types";

/** One composed frame of the open recording, at a source position. */
export type RecordingFrameRequest = {
  artifactId: number;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  keyboardEffects: KeyboardEffectSettings;
  positionMs: number;
  recordingOutput: RecordingOutputSettings;
  annotationClips?: import("./annotations/recording-annotations").RecordingAnnotationClip[];
};

const frame = ({
  annotationClips,
  artifactId,
  bakeCamera,
  cameraOverlay,
  cursorEffects,
  keyboardEffects,
  positionMs,
  recordingOutput,
}: RecordingFrameRequest) => ({
  annotationClips,
  artifactId,
  bakeCamera,
  cameraOverlay: normalizedCameraOverlay(
    cameraOverlay,
    recordingOutput.primary,
  ),
  cursorEffects: normalizedCursorEffects(cursorEffects),
  keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
  positionMs: Math.max(0, Math.round(positionMs)),
  recordingOutput: {
    camera: normalizedScreenshotOutput(recordingOutput.camera),
    primary: normalizedScreenshotOutput(recordingOutput.primary),
  },
});

export const copyRecordingPreviewFrameToClipboard = (
  request: RecordingFrameRequest,
) =>
  invoke<null>("copy_recording_preview_frame_to_clipboard", {
    frame: frame(request),
  });

/** Hands the app the edit the project browser's pictures of this recording
 * owe: the still at `stillPositionMs`, and the scrub strip at each of
 * `stripPositionsMs`, in order along the edit. The app makes them once the
 * edit rests, or before letting the recording go if the window closes
 * first. */
export const requestRecordingProjectPictures = ({
  request,
  stillPositionMs,
  stripPositionsMs,
}: {
  request: RecordingFrameRequest;
  stillPositionMs: number;
  stripPositionsMs: number[];
}) =>
  invoke<null>("request_recording_project_pictures", {
    frame: frame(request),
    stillPositionMs: Math.max(0, Math.round(stillPositionMs)),
    stripPositionsMs: stripPositionsMs.map((position) =>
      Math.max(0, Math.round(position)),
    ),
  });
