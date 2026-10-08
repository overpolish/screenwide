// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AudioTrackVolume } from "../../bindings/AudioTrackVolume";
import type { CameraOverlaySettings } from "../../bindings/CameraOverlaySettings";
import type { CursorEffectSettings } from "../../bindings/CursorEffectSettings";
import type { EditorArtifact } from "../../bindings/EditorArtifact";
import type { EditorKind } from "../../bindings/EditorKind";
import type { EditorSnapshot } from "../../bindings/EditorSnapshot";
import type { EditorSnapshots } from "../../bindings/EditorSnapshots";
import type { KeyboardEffectAnimation } from "../../bindings/KeyboardEffectAnimation";
import type { KeyboardEffectAppearance } from "../../bindings/KeyboardEffectAppearance";
import type { KeyboardEffectSettings } from "../../bindings/KeyboardEffectSettings";
import type { PreparedAudioTrack } from "../../bindings/PreparedAudioTrack";
import type { RecordingExportChoices } from "../../bindings/RecordingExportChoices";
import type { RecordingPreview } from "../../bindings/RecordingPreview";
import type { RecordingPreviewLayout } from "../../bindings/RecordingPreviewLayout";
import type { RecordingPreviewPane } from "../../bindings/RecordingPreviewPane";
import type { RecordingProjectLook } from "../../bindings/RecordingProjectLook";
import type { RememberedCameraOverlay } from "../../bindings/RememberedCameraOverlay";

export type {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  EditorArtifact,
  EditorKind,
  EditorSnapshot,
  EditorSnapshots,
  KeyboardEffectAnimation,
  KeyboardEffectAppearance,
  KeyboardEffectSettings,
  PreparedAudioTrack,
  RecordingExportChoices,
  RecordingPreview,
  RecordingPreviewLayout,
  RecordingPreviewPane,
  RecordingProjectLook,
  RememberedCameraOverlay,
};

export type RecordingVideoTrackId = "camera" | "primary";
export type RecordingTrackId = RecordingVideoTrackId | `audio:${number}`;
export type RecordingTimelineThumbnail = {
  id: string;
  url: string | null;
};
export type RecordingTimelineThumbnails = Record<
  RecordingVideoTrackId,
  RecordingTimelineThumbnail[]
>;

export type RecordingKeyboardTimelineItem = {
  endMs: number;
  id: number;
  label: string;
  startMs: number;
};

export const recordingAudioTrackId = (streamIndex: number): RecordingTrackId =>
  `audio:${String(streamIndex)}` as RecordingTrackId;

export const recordingAudioStreamIndex = (trackId: RecordingTrackId | null) => {
  if (!trackId?.startsWith("audio:")) return null;
  const streamIndex = Number(trackId.slice("audio:".length));
  return Number.isInteger(streamIndex) ? streamIndex : null;
};

export const initialEditorSnapshot = (
  workspace: EditorKind,
): EditorSnapshot => ({
  artifact: null,
  cursorEffects: {
    bake: true,
    clickAnimation: true,
    clipAtVideoEdge: false,
    motionBlur: true,
    sizePercent: 100,
    smoothMovement: true,
  },
  directory: null,
  keyboardEffects: {
    animation: "pop",
    appearance: "light",
    bake: true,
    sizePercent: 100,
  },
  recordingExportChoices: {
    bakeCamera: null,
    cameraCompression: null,
    cameraOverlay: null,
    cameraResolutionScalePercent: null,
    collapseAudio: null,
    compression: null,
    deleteProjectAfterExport: null,
    resolutionScaleRatio: null,
  },
  recordingOutput: null,
  screenshotBackgroundRadiusPercent: 0,
  screenshotDeleteProjectAfterExport: null,
  screenshotOutput: null,
  screenshotRadiusPercent: 0,
  workspace,
});
