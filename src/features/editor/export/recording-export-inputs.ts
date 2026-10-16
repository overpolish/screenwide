// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingProcessingOptions } from "../api";
import {
  defaultScreenshotOutput,
  RecordingOutputSettings,
} from "../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  EditorArtifact,
  KeyboardEffectSettings,
} from "../types";

import { VideoExportSettings } from "./recording-export-settings";

/** What the editor knows of a recording's export: what the size estimate
 * and the preview frame are made from. */
export type RecordingExportInputs = {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  camera: VideoExportSettings;
  cameraOverlay: CameraOverlaySettings;
  collapseAudio: boolean;
  compression: number;
  cursorEffects: CursorEffectSettings;
  enabledStreamIndices: number[] | null;
  includeCamera: boolean;
  includePrimaryVideo: boolean;
  keyboardEffects: KeyboardEffectSettings;
  recordingOutput: RecordingOutputSettings;
  recordingTimelineEdit: RecordingTimelineEdit | null;
  resolutionScalePercent: number;
};

/** `inputs` as the export's options. A recording has no screenshot canvas,
 * so it sends an empty one. */
export const recordingProcessingOptions = (
  inputs: RecordingExportInputs,
): RecordingProcessingOptions => ({
  audioTrackVolumes: inputs.audioTrackVolumes,
  bakeCamera: inputs.bakeCamera,
  cameraCompression: inputs.camera.compression,
  cameraOverlay: inputs.cameraOverlay,
  cameraResolutionScalePercent: inputs.camera.resolutionScalePercent,
  collapseAudio: inputs.collapseAudio,
  compression: inputs.compression,
  cursorEffects: inputs.cursorEffects,
  enabledStreamIndices: inputs.enabledStreamIndices ?? [],
  includeCamera: inputs.includeCamera,
  includePrimaryVideo: inputs.includePrimaryVideo,
  keyboardEffects: inputs.keyboardEffects,
  recordingOutput: inputs.recordingOutput,
  resolutionScalePercent: inputs.resolutionScalePercent,
  screenshotOutput: { ...defaultScreenshotOutput(1, 1), items: [] },
  timelineEdit: inputs.recordingTimelineEdit,
});
