// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingOutputSettings } from "../screenshot/screenshot-output";
import { EditorArtifact } from "../types";

import { CameraOutput } from "./camera-output";

import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";

export type EditorExportFormProps = {
  artifact: EditorArtifact | null;
  directory: string | null;
  /** What saving delivers, which is not always `artifact.extension`. */
  extension: string;
  fileStem: string;
  onBrowse: () => void;
  /** Dismisses without saving. The host owns what that means for its shell. */
  onCancel: () => void;
  onExport: () => void;
  onFileStemChange: (fileStem: string) => void;
  cameraCompression?: number;
  /** One video or separate files, null where there is no choice to make. */
  cameraOutput?: CameraOutput | null;
  cameraResolutionScalePercent?: number;
  canExport?: boolean;
  collapseAudio?: boolean;
  compression?: number;
  /** Moves the recording's project to the Trash once it has been saved. */
  deleteProjectAfterExport?: boolean;
  enabledAudioTrackCount?: number;
  estimatedSizeBytes?: number | null;
  /** Whether the recording has a transcript to make subtitles from. */
  hasTranscript?: boolean;
  /** Whether the camera is one of the enabled video tracks. */
  includeCamera?: boolean;
  isEstimatingSize?: boolean;
  isSaving?: boolean;
  onCameraCompressionChange?: (compression: number) => void;
  onCameraResolutionScaleChange?: (scale: number) => void;
  onCollapseAudioChange?: (collapse: boolean) => void;
  onCompressionChange?: (compression: number) => void;
  onDeleteProjectAfterExportChange?: (remove: boolean) => void;
  onOpenLocationAfterExportChange?: (open: boolean) => void;
  onResolutionScaleChange?: (scale: number) => void;
  onSubtitlesChange?: (subtitles: ExportSubtitles) => void;
  /** Reveals the saved file once it has been written. A stored preference. */
  openLocationAfterExport?: boolean;
  /** The canvases the editor's Dimensions setting gives the video and the
   * camera, which is what the size choices scale; absent, the source sizes
   * stand in. */
  recordingOutput?: RecordingOutputSettings | null;
  resolutionScalePercent?: number;
  /** The subtitle file written beside a recording's video. */
  subtitles?: ExportSubtitles;
};
