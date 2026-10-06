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

/** Keeps the frame, smaller, as the project browser's still for it. */
export const saveRecordingProjectStill = (request: RecordingFrameRequest) =>
  invoke<null>("save_recording_project_still", { frame: frame(request) });
