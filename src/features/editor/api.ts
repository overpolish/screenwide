// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke } from "@tauri-apps/api/core";

import {
  normalizedCameraOverlay,
  normalizedKeyboardEffects,
  normalizedAudioTrackVolumes,
} from "./preview/preview-settings-normalization";
import {
  RecordingPreviewPlayerEvent,
  RecordingPreviewPlayerInfo,
} from "./recording/recording-preview-player-contract";
import {
  normalizedScreenshotOutput,
  RecordingOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  normalizedScreenshotWorkspaceOutput,
} from "./screenshot/screenshot-output";
import { RecordingTimelineEdit } from "./timeline/editing/recording-timeline-edit";
import {
  CameraOverlaySettings,
  AudioTrackVolume,
  EditorSnapshots,
  RecordingPreview,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "./types";

import type { RecordingFormatPayload } from "./export/recording-format";
import type { CameraKey } from "../../bindings/CameraKey";
import type { ExportedFile } from "../../bindings/ExportedFile";
import type { ExportSubtitles } from "../../bindings/ExportSubtitles";
import type { ScreenshotFormat } from "../../bindings/ScreenshotFormat";

export type PreviewSelectionLayout = {
  paneIndex: number;
  radiusPercent: number;
  rect: { height: number; width: number; x: number; y: number };
  /** The crop keeps its shape as its handles are dragged. */
  cropKeepsShape?: boolean;
  cropMode?: boolean;
  /** How far the crop's chrome has taken over from the select tool's, 0 to
   * 1, as the crop opens and closes. */
  cropOpen?: number;
  image?: { height: number; width: number; x: number; y: number };
  isBlank?: boolean;
  layerId?: number;
  maximumScale?: number;
  minimumScale?: number;
  recenterBounds?: { height: number; width: number; x: number; y: number };
};
export const getEditorSnapshot = () =>
  invoke<EditorSnapshots>("get_editor_snapshot");
export const getRecordingPreview = (artifactId: number) =>
  invoke<RecordingPreview>("get_recording_preview", { artifactId });

export const startRecordingPreviewPlayer = ({
  artifactId,
  audioTrackVolumes,
  bakeCamera,
  cameraOverlay,
  cursorEffects,
  enabledStreamIndices,
  eventChannel,
  keyboardEffects,
  keyboardTimeline,
  recordingOutput,
  sessionId,
}: {
  artifactId: number;
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  enabledStreamIndices: number[];
  eventChannel: Channel<RecordingPreviewPlayerEvent>;
  keyboardEffects: KeyboardEffectSettings;
  keyboardTimeline: import("./timeline/keyboard/recording-keyboard-timeline-api").RecordingPreviewKeyboardDeletions;
  recordingOutput: RecordingOutputSettings;
  sessionId: number;
}) =>
  invoke<RecordingPreviewPlayerInfo>("start_recording_preview_player", {
    artifactId,
    eventChannel,
    sessionId,
    settings: {
      audio: {
        audioTrackVolumes: normalizedAudioTrackVolumes(audioTrackVolumes),
        enabledStreamIndices,
      },
      bakeCamera,
      cameraOverlay: normalizedCameraOverlay(
        cameraOverlay,
        recordingOutput.primary,
      ),
      cursorEffects,
      keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
      ...keyboardTimeline,
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
    },
  });

export const pauseRecordingPreview = (sessionId: number) =>
  invoke<null>("pause_recording_preview", { sessionId });

export const seekRecordingPreview = ({
  annotationClips,
  positionMs,
  requestId,
  rough = false,
  sessionId,
}: {
  positionMs: number;
  requestId: number;
  sessionId: number;
  annotationClips?: RecordingTimelineEdit["annotationClips"];
  rough?: boolean;
}) =>
  invoke<null>("seek_recording_preview", {
    annotationClips,
    positionMs: Number.isFinite(positionMs)
      ? Math.max(0, Math.round(positionMs))
      : 0,
    requestId,
    rough,
    sessionId,
  });

export const selectRecordingPreviewAudio = (
  enabledStreamIndices: number[],
  sessionId: number,
) =>
  invoke<null>("select_recording_preview_audio", {
    enabledStreamIndices,
    sessionId,
  });

export const setRecordingPreviewAudioVolumes = (
  audioTrackVolumes: AudioTrackVolume[],
  sessionId: number,
) =>
  invoke<null>("set_recording_preview_audio_volumes", {
    audioTrackVolumes: normalizedAudioTrackVolumes(audioTrackVolumes),
    sessionId,
  });

export const setRecordingPreviewCursorEffects = (
  cursorEffects: CursorEffectSettings,
  sessionId: number,
) =>
  invoke<null>("set_recording_preview_cursor_effects", {
    cursorEffects,
    sessionId,
  });

export const setRecordingPreviewKeyboardEffects = (
  keyboardEffects: KeyboardEffectSettings,
  sessionId: number,
) =>
  invoke<null>("set_recording_preview_keyboard_effects", {
    keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
    sessionId,
  });

export const setRecordingPreviewComposition = ({
  bakeCamera,
  cameraOverlay,
  recordingOutput,
  sessionId,
}: {
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  recordingOutput: RecordingOutputSettings;
  sessionId: number;
}) =>
  invoke<null>("set_recording_preview_composition", {
    bakeCamera,
    cameraOverlay: normalizedCameraOverlay(
      cameraOverlay,
      recordingOutput.primary,
    ),
    recordingOutput: {
      camera: normalizedScreenshotOutput(recordingOutput.camera),
      primary: normalizedScreenshotOutput(recordingOutput.primary),
    },
    sessionId,
  });

export { copyRecordingPreviewFrameToClipboard } from "./recording/recording-frame-api";

export const stopRecordingPreviewPlayer = (sessionId: number) =>
  invoke<null>("stop_recording_preview_player", { sessionId });

export const streamRecordingTimelineThumbnails = (
  artifactId: number,
  count: number,
  channel: Channel<ArrayBuffer>,
) =>
  invoke<null>("stream_recording_timeline_thumbnails", {
    artifactId,
    channel,
    count,
  });

export type RecordingProcessingOptions = {
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraCompression: number;
  cameraOverlay: CameraOverlaySettings;
  cameraResolutionScalePercent: number;
  collapseAudio: boolean;
  compression: number;
  cursorEffects: CursorEffectSettings;
  enabledStreamIndices: number[];
  includeCamera: boolean;
  includePrimaryVideo: boolean;
  keyboardEffects: KeyboardEffectSettings;
  recordingOutput: RecordingOutputSettings;
  resolutionScalePercent: number;
  screenshotOutput: ScreenshotWorkspaceOutputSettings;
  timelineEdit?: RecordingTimelineEdit | null;
};
/** The options of a recording's export as the export and the compared
 * frame read them, normalized as Rust expects. */
export const recordingProcessingPayload = (
  options: RecordingProcessingOptions,
) => ({
  ...options,
  keyboardEffects: normalizedKeyboardEffects(options.keyboardEffects),
  recordingOutput: {
    camera: normalizedScreenshotOutput(options.recordingOutput.camera),
    primary: normalizedScreenshotOutput(options.recordingOutput.primary),
  },
  screenshotOutput: normalizedScreenshotWorkspaceOutput(
    options.screenshotOutput,
  ),
});
// The format, with a GIF's stretch, travels in `processing`.
type SaveExportOptions = RecordingProcessingOptions &
  RecordingFormatPayload & {
    /** What fills a separate camera file where its background is taken away. */
    cameraKey: CameraKey;
    /** Moves the recording's project to the Trash once the export is saved. */
    deleteProjectAfterExport: boolean;
    fileStem: string;
    screenshotFormat: ScreenshotFormat;
    /** The subtitle file written beside a recording's video. */
    subtitles: ExportSubtitles;
  };

export const saveExport = ({
  cameraKey,
  deleteProjectAfterExport,
  fileStem,
  screenshotFormat,
  subtitles,
  ...processing
}: SaveExportOptions) =>
  invoke<ExportedFile | null>("save_export", {
    fileStem,
    options: {
      ...recordingProcessingPayload(processing),
      cameraKey,
      deleteProjectAfterExport,
      screenshotFormat,
      subtitles,
    },
  });

export {
  browseBackgroundImage,
  browseExportDirectory,
  cancelExportJob,
  copyEditorToClipboard,
  setExportDirectory,
} from "./editor-actions";

export const setRecordingPreviewEditorSuspended = (
  sessionId: number,
  suspended: boolean,
) =>
  invoke<null>("set_recording_preview_editor_suspended", {
    sessionId,
    suspended,
  });
