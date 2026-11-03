// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { exportLookQualities } from "./editor-export";
import { useExportIslandStore } from "./export-island-store";
import { useExportCompare } from "./use-export-compare";
import { useRecordingExportEstimate } from "./use-recording-export-estimate";

import type { CameraOutput } from "./camera-output";
import type { RecordingExportInputs } from "./recording-export-inputs";
import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot/screenshot-output";

/** What the Export island measures and draws of the picture while it is
 * out: the looks with each one's size, and the compare. Both are of the
 * picture, which the sound alone leaves out, so neither is made while it is
 * chosen. */
export function useExportMeasures(
  inputs: RecordingExportInputs,
  {
    cameraOutput,
    recordingFormat,
    screenshot,
  }: {
    cameraOutput: CameraOutput | null;
    recordingFormat: RecordingFormat;
    screenshot: {
      format: ScreenshotFormat;
      output: ScreenshotWorkspaceOutputSettings;
    };
  },
) {
  const isIslandOpen = useExportIslandStore((state) => state.isOpen);
  const isOpen =
    isIslandOpen &&
    (inputs.artifact?.kind !== "recording" || recordingFormat === "mp4");
  const lookQualities = exportLookQualities(inputs.artifact, cameraOutput);
  const lookSizeBytes = useRecordingExportEstimate(inputs, {
    isOpen,
    looks: lookQualities,
  });
  const exportCompare = useExportCompare(inputs, { isOpen, screenshot });
  return { exportCompare, lookQualities, lookSizeBytes };
}
