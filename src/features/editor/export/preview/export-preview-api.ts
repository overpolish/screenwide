// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  RecordingProcessingOptions,
  recordingProcessingPayload,
} from "../../api";

import type { ExportPreviewFrame } from "../../../../bindings/ExportPreviewFrame";
import type { ExportPreviewSide } from "../../../../bindings/ExportPreviewSide";

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
