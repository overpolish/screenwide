// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  cameraResolutionScales,
  resolutionScales,
  scaledDimensions,
  scaledVideoDimensions,
} from "../resolution";
import { RecordingOutputSettings } from "../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import { EditorArtifact } from "../types";

import { CameraOutput } from "./camera-output";
import {
  EXPORT_LOOKS,
  lookQuality,
  qualityLook,
  VideoQuality,
} from "./export-looks";

import type { CameraKey } from "../../../bindings/CameraKey";
import type { ExportedFile } from "../../../bindings/ExportedFile";
import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type {
  ToolPanelExport,
  ToolPanelExportCompare,
  ToolPanelExportPatch,
} from "../tools/tool-panel-export";

/** The export's settings as the editor window holds them. */
export type EditorExportSettings = {
  cameraCompression: number;
  cameraKey: CameraKey;
  cameraResolutionScalePercent: number;
  collapseAudio: boolean;
  compression: number;
  deleteProjectAfterExport: boolean;
  directory: string | null;
  fileStem: string;
  isCancelingSave: boolean;
  isSaving: boolean;
  resolutionScalePercent: number;
  screenshotFormat: ScreenshotFormat;
  subtitles: ExportSubtitles;
};

/** What the editor window does when the Export island asks. */
export type EditorExportHandlers = {
  onBrowse: () => void;
  onCameraCompressionChange: (compression: number) => void;
  onCameraKeyChange: (key: CameraKey) => void;
  onCameraOutputChange: (output: CameraOutput) => void;
  onCameraResolutionScaleChange: (scale: number) => void;
  onCancelSave: () => void;
  onCollapseAudioChange: (collapse: boolean) => void;
  onCompressionChange: (compression: number) => void;
  onDeleteProjectAfterExportChange: (remove: boolean) => void;
  onExport: () => void;
  onFileStemChange: (fileStem: string) => void;
  onResolutionScaleChange: (scale: number) => void;
  onScreenshotFormatChange: (format: ScreenshotFormat) => void;
  onSubtitlesChange: (subtitles: ExportSubtitles) => void;
};

type Recording = Extract<EditorArtifact, { kind: "recording" }>;

/** The scales the screen, and a separate camera where there is one, are
 * offered at, the original first: what the looks are made of. */
const exportScales = (
  recording: Recording,
  cameraOutput: CameraOutput | null,
) => ({
  camera: cameraOutput === "separate" ? cameraResolutionScales : null,
  screen: resolutionScales(recording),
});

/** What each look sets, in look order: the screen's quality, and a
 * separate camera's; none where the recording has no picture to compress. */
export type ExportLookQuality = {
  camera: VideoQuality | null;
  screen: VideoQuality;
};

export const exportLookQualities = (
  artifact: EditorArtifact | null,
  cameraOutput: CameraOutput | null,
): ExportLookQuality[] => {
  if (artifact?.kind !== "recording" || !hasQuality(artifact)) return [];
  const scales = exportScales(artifact, cameraOutput);
  return EXPORT_LOOKS.map((look) => ({
    camera: scales.camera && lookQuality(look, scales.camera),
    screen: lookQuality(look, scales.screen),
  }));
};

const hasQuality = (recording: Recording) =>
  recording.primaryKind !== "audio" && recording.canCompress;

/** Whether the camera's background is taken away anywhere in the edit. */
const removesCameraBackground = (edit: RecordingTimelineEdit | null) =>
  (edit?.cameraBackgroundRanges ?? []).some(
    (range) => range.background === "transparent",
  );

/** What the Export island shows, from what the editor holds; the general
 * setting it shows is added where the general settings are read
 * (`useEditorToolPanels`). */
export type EditorExportSnapshot = Omit<ToolPanelExport, "revealWhenDone">;

/** What the Export island shows, from what the editor holds. */
export function exportIslandSnapshot({
  artifact,
  cameraOutput,
  canExport,
  compare,
  enabledAudioTrackCount,
  exported,
  failure,
  hasTranscript,
  isAudioOnly,
  lookSizeBytes,
  recordingOutput,
  recordingTimelineEdit,
  settings,
}: {
  artifact: EditorArtifact | null;
  cameraOutput: CameraOutput | null;
  canExport: boolean;
  compare: ToolPanelExportCompare | null;
  enabledAudioTrackCount: number;
  exported: ExportedFile | null;
  failure: string | null;
  hasTranscript: boolean;
  isAudioOnly: boolean;
  /** Each look's measured size, in look order. */
  lookSizeBytes: readonly (number | null)[];
  recordingOutput: RecordingOutputSettings | null;
  recordingTimelineEdit: RecordingTimelineEdit | null;
  settings: EditorExportSettings;
}): EditorExportSnapshot {
  const recording = artifact?.kind === "recording" ? artifact : null;
  const kind = recording ? (isAudioOnly ? "audio" : "video") : "screenshot";
  const extension =
    kind === "screenshot"
      ? settings.screenshotFormat === "jpeg"
        ? "jpg"
        : "png"
      : kind === "audio"
        ? "m4a"
        : (artifact?.extension ?? "mp4");
  const name = exported?.path.split(/[\\/]/).pop() ?? "";
  return {
    cameraKey:
      cameraOutput === "separate" &&
      removesCameraBackground(recordingTimelineEdit)
        ? settings.cameraKey
        : null,
    cameraOutput,
    canExport,
    collapseAudio:
      recording && enabledAudioTrackCount > 1 ? settings.collapseAudio : null,
    compare: kind === "audio" ? null : compare,
    deleteProjectAfterExport: settings.deleteProjectAfterExport,
    directory: settings.directory,
    exported: exported && { bytes: exported.bytes, name },
    extension,
    failure,
    fileStem: settings.fileStem,
    kind,
    progress: settings.isSaving
      ? {
          // A screenshot is written in one pass the backend cannot stop.
          canCancel: recording !== null,
          etaSeconds: null,
          isCanceling: settings.isCancelingSave,
          percent: null,
          phase: "recording",
        }
      : null,
    quality:
      recording && kind === "video" && hasQuality(recording)
        ? exportQuality({
            cameraOutput,
            lookSizeBytes,
            recording,
            recordingOutput,
            settings,
          })
        : null,
    screenshotFormat: recording ? null : settings.screenshotFormat,
    subtitles: recording && hasTranscript ? settings.subtitles : null,
  };
}

function exportQuality({
  cameraOutput,
  lookSizeBytes,
  recording,
  recordingOutput,
  settings,
}: {
  cameraOutput: CameraOutput | null;
  lookSizeBytes: readonly (number | null)[];
  recording: Recording;
  recordingOutput: RecordingOutputSettings | null;
  settings: EditorExportSettings;
}): NonNullable<ToolPanelExport["quality"]> {
  const scales = exportScales(recording, cameraOutput);
  // The sizes are those of the canvas the Frame island's dimensions give
  // the video, where it has one; the source's size stands in before then.
  const output = recordingOutput?.primary;
  const dimensions = (scale: number) =>
    output
      ? scaledVideoDimensions({
          height: output.height,
          scale,
          sourceScale: scales.screen[0],
          width: output.width,
        })
      : scaledDimensions(recording, scale);
  return {
    look: qualityLook(
      {
        quality: {
          compression: settings.compression,
          resolutionScalePercent: settings.resolutionScalePercent,
        },
        scales: scales.screen,
      },
      scales.camera
        ? {
            quality: {
              compression: settings.cameraCompression,
              resolutionScalePercent: settings.cameraResolutionScalePercent,
            },
            scales: scales.camera,
          }
        : null,
    ),
    looks: EXPORT_LOOKS.map((id, index) => ({
      bytes: lookSizeBytes[index] ?? null,
      ...dimensions(lookQuality(id, scales.screen).resolutionScalePercent),
      id,
    })),
  };
}

/** Does what the Export island asks, through the editor window's own
 * handlers, so the coupling between compression and size holds as it does
 * everywhere: a look sets the size first, then the compression. */
export function applyExportPatch(
  patch: ToolPanelExportPatch,
  on: EditorExportHandlers,
  looks: ExportLookQuality[],
) {
  if (patch.fileStem !== undefined) on.onFileStemChange(patch.fileStem);
  if (patch.browse) on.onBrowse();
  if (patch.screenshotFormat)
    on.onScreenshotFormatChange(patch.screenshotFormat);
  const look = patch.look && looks[EXPORT_LOOKS.indexOf(patch.look)];
  if (look) {
    on.onResolutionScaleChange(look.screen.resolutionScalePercent);
    on.onCompressionChange(look.screen.compression);
    if (look.camera) {
      on.onCameraResolutionScaleChange(look.camera.resolutionScalePercent);
      on.onCameraCompressionChange(look.camera.compression);
    }
  }
  if (patch.subtitles) on.onSubtitlesChange(patch.subtitles);
  if (patch.cameraOutput) on.onCameraOutputChange(patch.cameraOutput);
  if (patch.cameraKey) on.onCameraKeyChange(patch.cameraKey);
  if (patch.collapseAudio !== undefined)
    on.onCollapseAudioChange(patch.collapseAudio);
  if (patch.deleteProjectAfterExport !== undefined)
    on.onDeleteProjectAfterExportChange(patch.deleteProjectAfterExport);
  if (patch.cancel) on.onCancelSave();
  if (patch.export) on.onExport();
}
