// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke } from "@tauri-apps/api/core";

import { AnnotationStyle, ImageArt } from "./annotations/annotations";
import { type ViewportCorners } from "./preview/preview-backdrop";
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

import type { CameraKey } from "../../bindings/CameraKey";
import type { ExportedFile } from "../../bindings/ExportedFile";
import type { ExportLookQuality } from "../../bindings/ExportLookQuality";
import type { ExportSubtitles } from "../../bindings/ExportSubtitles";
import type { ScreenshotFormat } from "../../bindings/ScreenshotFormat";
import type { TextStyle } from "../../bindings/TextStyle";

type PreviewSelectionLayout = {
  paneIndex: number;
  radiusPercent: number;
  rect: { height: number; width: number; x: number; y: number };
  /** The crop keeps its shape as its handles are dragged. */
  cropKeepsShape?: boolean;
  cropMode?: boolean;
  image?: { height: number; width: number; x: number; y: number };
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

export const layoutRecordingPreviewSurface = ({
  annotationTool,
  backdrop,
  bakeCamera,
  cameraOverlay,
  nativeEditor,
  panes,
  recordingOutput,
  requestId,
  scale,
  selection,
  selectionTargets,
  sessionId,
  viewport,
  viewportCorners,
}: {
  backdrop: [number, number, number, number];
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  nativeEditor: boolean;
  panes: {
    index: number;
    rect: { height: number; width: number; x: number; y: number };
  }[];
  recordingOutput: RecordingOutputSettings;
  requestId: number;
  scale: number;
  sessionId: number;
  viewport: { height: number; width: number; x: number; y: number };
  /** How round the viewport's corners are where it meets the chrome. */
  viewportCorners: ViewportCorners;
  /** The annotation tool in hand, when one is. It rides with the layout so
   * the tool and the selection it has to agree with reach the native chrome
   * together: split across two commands, whichever lands first paints the
   * layer's frame for a frame. */
  annotationTool?:
    import("./annotations/annotation-defaults").AnnotationTool | null;
  selection?: PreviewSelectionLayout | null;
  selectionTargets?: PreviewSelectionLayout[] | null;
}) =>
  invoke<null>("layout_recording_preview_surface", {
    layout: {
      annotationTool: annotationTool ?? null,
      backdrop,
      bakeCamera,
      cameraOverlay: normalizedCameraOverlay(
        cameraOverlay,
        recordingOutput.primary,
      ),
      nativeEditor,
      panes,
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
      requestId,
      scale,
      selection: selection ?? null,
      selectionTargets: selectionTargets ?? null,
      sessionId,
      viewport,
      viewportCorners,
    },
  });

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

export const startScreenshotPreview = (artifactId: number, sessionId: number) =>
  invoke<null>("start_screenshot_preview", { artifactId, sessionId });

export const layoutScreenshotPreviewSurface = ({
  annotationCounterAngle,
  annotationDefaults,
  annotationImage,
  annotationTextLook,
  annotationTool,
  annotationZoom,
  backdrop,
  interactionOutput,
  nativeEditor,
  output,
  panes,
  scale,
  selectedAnnotationIds = [],
  selection,
  selectionTargets,
  sessionId,
  viewport,
  viewportCorners,
}: {
  backdrop: [number, number, number, number];
  interactionOutput: ScreenshotWorkspaceOutputSettings;
  nativeEditor: boolean;
  output: ScreenshotWorkspaceOutputSettings;
  panes: {
    index: number;
    rect: { height: number; width: number; x: number; y: number };
  }[];
  scale: number;
  sessionId: number;
  viewport: { height: number; width: number; x: number; y: number };
  /** How round the viewport's corners are where it meets the chrome. */
  viewportCorners: ViewportCorners;
  /** Where a fresh counter's tail points, in radians clockwise from east:
   * whatever the last counter was turned to. */
  annotationCounterAngle?: number | null;
  /** The dress the next fresh arrow is drawn in: whatever the editor's last
   * annotation edit settled on. Absent until it has settled on anything, and
   * the native tool falls back to the accent at its own stroke. */
  annotationDefaults?: AnnotationStyle | null;
  /** The picture a fresh image shows: whatever the last image was given. */
  annotationImage?: ImageArt | null;
  /** The look a fresh text is dressed in: whatever the last text was. */
  annotationTextLook?: TextStyle | null;
  /** The annotation tool in hand. "select" hit-tests the annotations already on
   * the layer and lets every other press fall through to it; "arrow" and
   * "counter" also make a new annotation on empty picture. */
  annotationTool?: import("./annotations/annotation-defaults").AnnotationTool;
  /** How many times a fresh lens enlarges: whatever the last lens was set
   * to. */
  annotationZoom?: number | null;
  /** Every annotation chosen on the selected layer. */
  selectedAnnotationIds?: readonly string[];
  selection?: PreviewSelectionLayout | null;
  selectionTargets?: PreviewSelectionLayout[] | null;
}) =>
  invoke<null>("layout_screenshot_preview_surface", {
    annotationCounterAngle: annotationCounterAngle ?? null,
    annotationDefaults: annotationDefaults ?? null,
    annotationImage: annotationImage ?? null,
    annotationTextLook: annotationTextLook ?? null,
    annotationTool: annotationTool ?? null,
    annotationZoom: annotationZoom ?? null,
    backdrop,
    interactionOutput: normalizedScreenshotWorkspaceOutput(interactionOutput),
    nativeEditor,
    output: normalizedScreenshotWorkspaceOutput(output),
    panes,
    scale,
    selectedAnnotationIds,
    selection: selection ?? null,
    selectionTargets: selectionTargets ?? null,
    sessionId,
    viewport,
    viewportCorners,
  });

export const refreshScreenshotPreviewSources = (
  artifactId: number,
  sessionId: number,
) =>
  invoke<null>("refresh_screenshot_preview_sources", {
    artifactId,
    sessionId,
  });

export const stopScreenshotPreview = (sessionId: number) =>
  invoke<null>("stop_screenshot_preview", { sessionId });

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
/** The options of a recording's export as the estimate and the compared
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
/** The recording's size, in bytes, at each of `looks`, in order. */
export const estimateRecordingExport = ({
  artifactId,
  looks,
  ...options
}: RecordingProcessingOptions & {
  artifactId: number;
  looks: ExportLookQuality[];
}) =>
  invoke<number[]>("estimate_recording_export", {
    artifactId,
    looks,
    options: recordingProcessingPayload(options),
  });
type SaveExportOptions = RecordingProcessingOptions & {
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
  audioTrackVolumes,
  bakeCamera,
  cameraCompression,
  cameraKey,
  cameraOverlay,
  cameraResolutionScalePercent,
  collapseAudio,
  compression,
  cursorEffects,
  deleteProjectAfterExport,
  enabledStreamIndices,
  fileStem,
  includeCamera,
  includePrimaryVideo,
  keyboardEffects,
  recordingOutput,
  resolutionScalePercent,
  screenshotFormat,
  screenshotOutput,
  subtitles,
  timelineEdit,
}: SaveExportOptions) =>
  invoke<ExportedFile | null>("save_export", {
    fileStem,
    options: {
      audioTrackVolumes,
      bakeCamera,
      cameraCompression,
      cameraKey,
      cameraOverlay,
      cameraResolutionScalePercent,
      collapseAudio,
      compression,
      cursorEffects,
      deleteProjectAfterExport,
      enabledStreamIndices,
      includeCamera,
      includePrimaryVideo,
      keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
      resolutionScalePercent,
      screenshotFormat,
      screenshotOutput: normalizedScreenshotWorkspaceOutput(screenshotOutput),
      subtitles,
      timelineEdit,
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

export const setScreenshotPreviewEditorSuspended = (
  sessionId: number,
  suspended: boolean,
) =>
  invoke<null>("set_screenshot_preview_editor_suspended", {
    sessionId,
    suspended,
  });
