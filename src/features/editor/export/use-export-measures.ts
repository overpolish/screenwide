// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { exportLookQualities } from "./editor-export";
import { useExportIslandStore } from "./export-island-store";
import { useExportCompare } from "./use-export-compare";

import type { CameraOutput } from "./camera-output";
import type { RecordingExportInputs } from "./recording-export-inputs";
import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot/screenshot-output";

/** What the Export island needs of the picture while it is out: what each
 * look sets, and the compare. The compare is of the picture, which the
 * sound alone leaves out, so it is not made while that is chosen. */
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
  const exportCompare = useExportCompare(inputs, { isOpen, screenshot });
  return { exportCompare, lookQualities };
}
