// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EXPORT_LOOKS } from "./export-looks";

import type { CameraOutput } from "./camera-output";
import type { ExportLookQuality } from "./editor-export";
import type { CameraKey } from "../../../bindings/CameraKey";
import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { ToolPanelExportPatch } from "../tools/tool-panel-export";

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
  onRecordingFormatChange: (format: RecordingFormat) => void;
  onResolutionScaleChange: (scale: number) => void;
  onScreenshotFormatChange: (format: ScreenshotFormat) => void;
  onSubtitlesChange: (subtitles: ExportSubtitles) => void;
};

/** Does what the Export island asks, through the editor window's own
 * handlers, so the coupling between compression and size holds as it does
 * everywhere: a look sets the size first, then the compression. */
export function applyExportPatch(
  patch: ToolPanelExportPatch,
  on: EditorExportHandlers,
  looks: ExportLookQuality[],
) {
  if (patch.fileStem !== undefined) on.onFileStemChange(patch.fileStem);
  if (patch.recordingFormat) on.onRecordingFormatChange(patch.recordingFormat);
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
