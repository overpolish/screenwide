// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke } from "@tauri-apps/api/core";

import {
  RecordingPreviewPlayerEvent,
  RecordingPreviewPlayerInfo,
} from "./recording-preview-player-contract";
import { RecordingTimelineEdit } from "./recording-timeline-edit";
import {
  normalizedScreenshotOutput,
  RecordingOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  normalizedScreenshotWorkspaceOutput,
} from "./screenshot-output";
import {
  CameraOverlaySettings,
  AudioTrackVolume,
  EditorSnapshots,
  RecordingPreview,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "./types";

const finite = (value: number, fallback: number) =>
  Number.isFinite(value) ? value : fallback;

/**
 * Guard the overlay against a value that is not a number.
 *
 * Its placement is in the screen output's own pixels, so the canvas is what a
 * fallback has to be measured against: the shares below are the ones the
 * overlay used to be written in.
 */
const normalizedCameraOverlay = (
  settings: CameraOverlaySettings,
  canvas: { height: number; width: number },
): CameraOverlaySettings => {
  const width = Math.max(1, canvas.width);
  const height = Math.max(1, canvas.height);
  return {
    cameraWidth: finite(settings.cameraWidth, width * 0.25),
    cameraX: finite(settings.cameraX, width * 0.85),
    cameraY: finite(settings.cameraY, height * 0.15),
    frameHeight: finite(settings.frameHeight, height * 0.25),
    frameWidth: finite(settings.frameWidth, width * 0.25),
    frameX: finite(settings.frameX, width * 0.72),
    frameY: finite(settings.frameY, height * 0.03),
    radiusPercent: finite(settings.radiusPercent, 8),
  };
};

const normalizedCursorEffects = (
  settings: CursorEffectSettings,
): CursorEffectSettings => ({
  ...settings,
  sizePercent: finite(settings.sizePercent, 100),
});

const normalizedKeyboardEffects = (
  settings: KeyboardEffectSettings,
): KeyboardEffectSettings => ({
  animation:
    settings.animation === "fade" || settings.animation === "none"
      ? settings.animation
      : "pop",
  appearance: settings.appearance === "dark" ? "dark" : "light",
  bake: settings.bake,
  positionXPercent: settings.positionXPercent,
  positionYPercent: settings.positionYPercent,
  sizePercent: finite(settings.sizePercent, 100),
});
const normalizedAudioTrackVolumes = (volumes: AudioTrackVolume[]) =>
  volumes.map((volume) => ({
    ...volume,
    decibels: finite(volume.decibels, 0),
  }));

type PreviewSelectionLayout = {
  paneIndex: number;
  radiusPercent: number;
  rect: { height: number; width: number; x: number; y: number };
  cropMode?: boolean;
  image?: { height: number; width: number; x: number; y: number };
  layerId?: number;
  maximumScale?: number;
  minimumScale?: number;
  recenterBounds?: { height: number; width: number; x: number; y: number };
  recenterMode?: boolean;
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
  keyboardTimeline: import("./recording-keyboard-timeline-api").RecordingPreviewKeyboardDeletions;
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
      cursorEffects: normalizedCursorEffects(cursorEffects),
      keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
      ...keyboardTimeline,
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        cameraOnTop: recordingOutput.cameraOnTop,
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
    },
  });

export const pauseRecordingPreview = (sessionId: number) =>
  invoke<null>("pause_recording_preview", { sessionId });

export const setRecordingPreviewZoom = (
  sessionId: number,
  zoomPercent: number,
) =>
  invoke<null>("set_recording_preview_zoom", {
    sessionId,
    zoomPercent,
  });

export const layoutRecordingPreviewSurface = ({
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
  selection?: PreviewSelectionLayout | null;
  selectionTargets?: PreviewSelectionLayout[] | null;
}) =>
  invoke<null>("layout_recording_preview_surface", {
    layout: {
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
        cameraOnTop: recordingOutput.cameraOnTop,
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
      requestId,
      scale,
      selection: selection ?? null,
      selectionTargets: selectionTargets ?? null,
      sessionId,
      viewport,
    },
  });

export const seekRecordingPreview = ({
  positionMs,
  requestId,
  rough = false,
  selectionVisible,
  sessionId,
}: {
  positionMs: number;
  requestId: number;
  sessionId: number;
  rough?: boolean;
  selectionVisible?: boolean;
}) =>
  invoke<null>("seek_recording_preview", {
    positionMs: Number.isFinite(positionMs)
      ? Math.max(0, Math.round(positionMs))
      : 0,
    requestId,
    rough,
    selectionVisible,
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
    cursorEffects: normalizedCursorEffects(cursorEffects),
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
      cameraOnTop: recordingOutput.cameraOnTop,
      primary: normalizedScreenshotOutput(recordingOutput.primary),
    },
    sessionId,
  });

export const copyRecordingPreviewFrameToClipboard = ({
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
}) =>
  invoke<null>("copy_recording_preview_frame_to_clipboard", {
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

export const startScreenshotPreview = (artifactId: number, sessionId: number) =>
  invoke<null>("start_screenshot_preview", { artifactId, sessionId });

export const layoutScreenshotPreviewSurface = ({
  backdrop,
  interactionOutput,
  nativeEditor,
  output,
  panes,
  scale,
  selection,
  selectionTargets,
  sessionId,
  viewport,
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
  selection?: PreviewSelectionLayout | null;
  selectionTargets?: PreviewSelectionLayout[] | null;
}) =>
  invoke<null>("layout_screenshot_preview_surface", {
    backdrop,
    interactionOutput: normalizedScreenshotWorkspaceOutput(interactionOutput),
    nativeEditor,
    output: normalizedScreenshotWorkspaceOutput(output),
    panes,
    scale,
    selection: selection ?? null,
    selectionTargets: selectionTargets ?? null,
    sessionId,
    viewport,
  });

export const refreshScreenshotPreviewSources = (
  artifactId: number,
  sessionId: number,
) =>
  invoke<null>("refresh_screenshot_preview_sources", {
    artifactId,
    sessionId,
  });

export const setScreenshotPreviewZoom = (
  sessionId: number,
  zoomPercent: number,
) =>
  invoke<null>("set_screenshot_preview_zoom", {
    sessionId,
    zoomPercent,
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

type RecordingProcessingOptions = {
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
export const estimateRecordingExport = ({
  artifactId,
  audioTrackVolumes,
  bakeCamera,
  cameraCompression,
  cameraOverlay,
  cameraResolutionScalePercent,
  collapseAudio,
  compression,
  cursorEffects,
  enabledStreamIndices,
  includeCamera,
  includePrimaryVideo,
  keyboardEffects,
  recordingOutput,
  resolutionScalePercent,
  screenshotOutput,
  timelineEdit,
}: RecordingProcessingOptions & { artifactId: number }) =>
  invoke<number>("estimate_recording_export", {
    artifactId,
    options: {
      audioTrackVolumes,
      bakeCamera,
      cameraCompression,
      cameraOverlay,
      cameraResolutionScalePercent,
      collapseAudio,
      compression,
      cursorEffects,
      enabledStreamIndices,
      includeCamera,
      includePrimaryVideo,
      keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        cameraOnTop: recordingOutput.cameraOnTop,
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
      resolutionScalePercent,
      screenshotOutput: normalizedScreenshotWorkspaceOutput(screenshotOutput),
      timelineEdit,
    },
  });
type SaveExportOptions = RecordingProcessingOptions & {
  fileStem: string;
};

export const saveExport = ({
  audioTrackVolumes,
  bakeCamera,
  cameraCompression,
  cameraOverlay,
  cameraResolutionScalePercent,
  collapseAudio,
  compression,
  cursorEffects,
  enabledStreamIndices,
  fileStem,
  includeCamera,
  includePrimaryVideo,
  keyboardEffects,
  recordingOutput,
  resolutionScalePercent,
  screenshotOutput,
  timelineEdit,
}: SaveExportOptions) =>
  invoke<string | null>("save_export", {
    fileStem,
    options: {
      audioTrackVolumes,
      bakeCamera,
      cameraCompression,
      cameraOverlay,
      cameraResolutionScalePercent,
      collapseAudio,
      compression,
      cursorEffects,
      enabledStreamIndices,
      includeCamera,
      includePrimaryVideo,
      keyboardEffects: normalizedKeyboardEffects(keyboardEffects),
      recordingOutput: {
        camera: normalizedScreenshotOutput(recordingOutput.camera),
        cameraOnTop: recordingOutput.cameraOnTop,
        primary: normalizedScreenshotOutput(recordingOutput.primary),
      },
      resolutionScalePercent,
      screenshotOutput: normalizedScreenshotWorkspaceOutput(screenshotOutput),
      timelineEdit,
    },
  });

export {
  browseBackgroundImage,
  browseExportDirectory,
  cancelExportJob,
  copyEditorToClipboard,
  focusEditorWindow,
  setExportDirectory,
  setScreenshotBackgroundRadius,
  setScreenshotRadius,
} from "./api/editor-actions";

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

/**
 * Widens an editor window by `delta` logical px so an opening tool panel costs
 * the preview none of its usable width. Resolves to whether it grew: the
 * display may have no room to its right, and the picture then fits smaller
 * instead.
 */
export const growEditorForPanel = (label: string, delta: number) =>
  invoke<boolean>("grow_editor_for_panel", { delta, label });

/** Takes that width back, but only from a window still exactly as wide as the
 * growth left it: a resize in between is the size the user chose. */
export const shrinkEditorAfterPanel = (
  label: string,
  delta: number,
  expectedWidth: number,
) => invoke<null>("shrink_editor_after_panel", { delta, expectedWidth, label });
