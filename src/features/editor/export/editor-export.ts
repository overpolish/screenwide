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
import { recordingFormatChoice } from "./recording-format";

import type { CameraKey } from "../../../bindings/CameraKey";
import type { ExportedFile } from "../../../bindings/ExportedFile";
import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type {
  ToolPanelExport,
  ToolPanelExportCompare,
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
  recordingFormat: RecordingFormat;
  resolutionScalePercent: number;
  screenshotFormat: ScreenshotFormat;
  subtitles: ExportSubtitles;
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

/** The file extension each screenshot format is saved with, as Rust's
 * `ScreenshotFormat::extension` gives it. */
const SCREENSHOT_EXTENSIONS: Record<ScreenshotFormat, string> = {
  jpeg: "jpg",
  png: "png",
  webp: "webp",
};

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
  const recordingFormat =
    recording &&
    recordingFormatChoice(recording, {
      chosen: settings.recordingFormat,
      isAudioOnly,
    });
  const kind = recordingFormat
    ? recordingFormat.selected === "mp4"
      ? "video"
      : "audio"
    : "screenshot";
  const extension =
    kind === "screenshot"
      ? SCREENSHOT_EXTENSIONS[settings.screenshotFormat]
      : (recordingFormat?.offered.find(
          ({ format }) => format === recordingFormat.selected,
        )?.extension ?? "mp4");
  // The sound alone carries no picture, so nothing about the camera applies.
  const shownCameraOutput = kind === "video" ? cameraOutput : null;
  const name = exported?.path.split(/[\\/]/).pop() ?? "";
  return {
    cameraKey:
      shownCameraOutput === "separate" &&
      removesCameraBackground(recordingTimelineEdit)
        ? settings.cameraKey
        : null,
    cameraOutput: shownCameraOutput,
    canExport,
    // A WAV holds one stream, so its tracks are always mixed.
    collapseAudio:
      recording &&
      enabledAudioTrackCount > 1 &&
      recordingFormat?.selected !== "wav"
        ? settings.collapseAudio
        : null,
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
    recordingFormat,
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
