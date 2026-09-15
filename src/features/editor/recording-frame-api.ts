// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  normalizedCameraOverlay,
  normalizedCursorEffects,
  normalizedKeyboardEffects,
} from "./preview-settings-normalization";
import {
  normalizedScreenshotOutput,
  RecordingOutputSettings,
} from "./screenshot-output";
import {
  CameraOverlaySettings,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "./types";

export const copyRecordingPreviewFrameToClipboard = ({
  annotationClips,
  artifactId,
  bakeCamera,
  cameraOverlay,
  cursorEffects,
  keyboardEffects,
  positionMs,
  recordingOutput,
}: {
  artifactId: number;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  keyboardEffects: KeyboardEffectSettings;
  positionMs: number;
  recordingOutput: RecordingOutputSettings;
  annotationClips?: import("./recording-annotations").RecordingAnnotationClip[];
}) =>
  invoke<null>("copy_recording_preview_frame_to_clipboard", {
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
      cameraOnTop: recordingOutput.cameraOnTop,
      primary: normalizedScreenshotOutput(recordingOutput.primary),
    },
  });
