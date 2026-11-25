// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { recordingProcessingPayload } from "../api";
import {
  normalizedScreenshotOutput,
  normalizedScreenshotWorkspaceOutput,
} from "../screenshot/screenshot-output";

import type { RecordingFormatPayload } from "./recording-format";
import type { CameraKey } from "../../../bindings/CameraKey";
import type { ExportedFile } from "../../../bindings/ExportedFile";
import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { RecordingProcessingOptions } from "../api";
import type { ScreenshotSaveJob } from "../screenshot/formats/screenshot-save-jobs";
import type {
  RecordingOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot/screenshot-output";
import type { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import type { CameraOverlaySettings } from "../types";

// The format, with a GIF's stretch, travels in `processing`.
export type SaveExportOptions = RecordingProcessingOptions &
  RecordingFormatPayload & {
    /** What fills a separate camera file where its background is taken away. */
    cameraKey: CameraKey;
    /** Moves the recording's project to the Trash once the export is saved. */
    deleteProjectAfterExport: boolean;
    fileStem: string;
    screenshotFormat: ScreenshotFormat;
    screenshotFullColour: boolean;
    /** The subtitle file written beside a recording's video. */
    subtitles: ExportSubtitles;
    /** A screenshot's main canvas, remembered as the next screenshot's look
     * in place of the format in hand's. */
    screenshotTemplate?: ScreenshotWorkspaceOutputSettings;
  };

/** What one format does differently in an export of every format. */
export type FormatExport = {
  cameraOverlay: CameraOverlaySettings;
  fileStem: string;
  recordingOutput: RecordingOutputSettings;
  timelineEdit: RecordingTimelineEdit | null;
  /** Written as a copy of the format before it, which comes out the same. */
  isCopy?: boolean;
};

const exportPayload = ({
  cameraKey,
  deleteProjectAfterExport,
  screenshotFormat,
  screenshotFullColour,
  screenshotTemplate,
  subtitles,
  ...processing
}: Omit<SaveExportOptions, "fileStem">) => ({
  ...recordingProcessingPayload(processing),
  cameraKey,
  deleteProjectAfterExport,
  screenshotFormat,
  screenshotFullColour,
  screenshotTemplate: screenshotTemplate
    ? normalizedScreenshotWorkspaceOutput(screenshotTemplate)
    : null,
  subtitles,
});

const saveExport = ({ fileStem, ...options }: SaveExportOptions) =>
  invoke<ExportedFile | null>("save_export", {
    fileStem,
    options: exportPayload(options),
  });

/** Every format in one export, which draws them all from one decode of the
 * recording: `options` as they share them, their `fileStem` the export's own
 * name, which a camera saved as a file of its own takes, and `formats` what
 * each does differently, in the order their files are written. The file the
 * finished island shows, the last, or null where cancelled. */
const saveFormatExports = (
  { fileStem, ...options }: SaveExportOptions,
  formats: readonly FormatExport[],
) =>
  invoke<ExportedFile | null>("save_format_exports", {
    fileStem,
    formats: formats.map((format) => ({
      cameraOverlay: format.cameraOverlay,
      fileStem: format.fileStem,
      isCopy: format.isCopy ?? false,
      recordingOutput: {
        camera: normalizedScreenshotOutput(format.recordingOutput.camera),
        primary: normalizedScreenshotOutput(format.recordingOutput.primary),
      },
      timelineEdit: format.timelineEdit,
    })),
    options: exportPayload(options),
  });

/** Every format of a screenshot in one export: `options` as they share
 * them, and `jobs` each format's workspace and name, in the order their
 * files are written. The file the finished island shows, the last, or
 * null where cancelled. */
const saveScreenshotFormats = (
  { fileStem: _fileStem, ...options }: SaveExportOptions,
  jobs: readonly ScreenshotSaveJob[],
) =>
  invoke<ExportedFile | null>("save_screenshot_formats", {
    formats: jobs.map((job) => ({
      fileStem: job.fileStem,
      isCopy: job.isCopy ?? false,
      screenshotOutput: normalizedScreenshotWorkspaceOutput(job.workspace),
    })),
    options: exportPayload(options),
  });

/** `jobs` saved under the export's name `fileStem`, each with the options
 * `options` gives it: one alone, or every format in one export that draws
 * them all from one decode of the recording, a job that is a copy of the one
 * before written as a copy of its file. A screenshot made in several
 * formats writes `screenshots` instead, every format's file. The file the
 * finished island shows, or null where cancelled. */
export function saveFormats<Job extends FormatExport>(
  {
    fileStem,
    screenshots,
  }: { fileStem: string; screenshots: readonly ScreenshotSaveJob[] | null },
  jobs: readonly Job[],
  options: (job: Job) => SaveExportOptions,
): Promise<ExportedFile | null> {
  if (jobs.length === 0) return Promise.resolve(null);
  const first = jobs[0];
  if (screenshots) return saveScreenshotFormats(options(first), screenshots);
  if (jobs.length === 1) return saveExport(options(first));
  return saveFormatExports({ ...options(first), fileStem }, jobs);
}
