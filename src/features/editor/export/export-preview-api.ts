// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { RecordingProcessingOptions, recordingProcessingPayload } from "../api";
import { normalizedScreenshotWorkspaceOutput } from "../screenshot/screenshot-output";

import type { ExportPreviewFrame } from "../../../bindings/ExportPreviewFrame";
import type { ExportPreviewSide } from "../../../bindings/ExportPreviewSide";
import type { ScreenshotExportPreview } from "../../../bindings/ScreenshotExportPreview";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot/screenshot-output";

/** The frame `positionMs` into the recording as the export with `options`
 * writes it, or at Original; null where a newer frame replaced it first. */
export const previewRecordingExportFrame = ({
  artifactId,
  options,
  positionMs,
  side,
}: {
  artifactId: number;
  options: RecordingProcessingOptions;
  positionMs: number;
  side: ExportPreviewSide;
}) =>
  invoke<ExportPreviewFrame | null>("preview_recording_export_frame", {
    artifactId,
    options: recordingProcessingPayload(options),
    positionMs: Math.max(0, Math.round(positionMs)),
    side,
  });

/** The open screenshot as drawn and as its JPEG; null where a newer one
 * replaced it first. */
export const previewScreenshotExport = (
  screenshotOutput: ScreenshotWorkspaceOutputSettings,
) =>
  invoke<ScreenshotExportPreview | null>("preview_screenshot_export", {
    screenshotOutput: normalizedScreenshotWorkspaceOutput(screenshotOutput),
  });
