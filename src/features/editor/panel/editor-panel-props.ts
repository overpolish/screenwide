// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  EditorArtifact,
  KeyboardEffectSettings,
  PreparedAudioTrack,
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

import {
  RecordingOutputChange,
  ScreenshotOutputChange,
} from "./editor-content";

import type { CameraKey } from "../../../bindings/CameraKey";
import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { ExportLookQuality } from "../export/editor-export";
import type { ToolPanelExportCompare } from "../tools/tool-panel-export";

export type EditorPanelProps = {
  artifact: EditorArtifact | null;
  directory: string | null;
  fileStem: string;
  audioTrackVolumes?: AudioTrackVolume[];
  bakeCamera?: boolean;
  cameraCompression?: number;
  cameraKey?: CameraKey;
  cameraOverlay?: CameraOverlaySettings;
  cameraResolutionScalePercent?: number;
  collapseAudio?: boolean;
  compression?: number;
  cursorEffects?: CursorEffectSettings;
  deleteProjectAfterExport?: boolean;
  enabledAudioTrackCount?: number;
  enabledStreamIndices?: number[];
  enabledVideoTracks?: RecordingVideoTrackId[];
  /** What the Export island compares, or null where there is nothing. */
  exportCompare?: ToolPanelExportCompare | null;
  isCancelingSave?: boolean;
  isPreparingRecordingAudio?: boolean;
  isPreparingRecordingPreview?: boolean;
  isPreviewPreparing?: boolean;
  isSaving?: boolean;
  keyboardEffects?: KeyboardEffectSettings;
  /** What each of the Export island's looks sets. */
  lookQualities?: ExportLookQuality[];
  /** Draws the camera into the video, or saves it as a file of its own. */
  onBakeCameraChange?: (bake: boolean) => void;
  onBrowse?: () => void;
  onCameraCompressionChange?: (compression: number) => void;
  onCameraKeyChange?: (key: CameraKey) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onCameraResolutionScaleChange?: (scale: number) => void;
  onCancel?: () => void;
  onCancelSave?: () => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCollapseAudioChange?: (collapse: boolean) => void;
  onCompressionChange?: (compression: number) => void;
  onCopy?: () => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  /** Closes the editor and sets its project aside in Recently Deleted. */
  onDeleteProject?: () => void;
  onDeleteProjectAfterExportChange?: (remove: boolean) => void;
  onEnabledTracksChange?: (streamIndices: number[]) => void;
  onEnabledVideoTracksChange?: (tracks: RecordingVideoTrackId[]) => void;
  onFileStemChange?: (fileStem: string) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onMinimize?: () => void;
  onRecordingFormatChange?: (format: RecordingFormat) => void;
  onRecordingOutputChange?: RecordingOutputChange;
  onRecordingTimelineEditChange?: (edit: RecordingTimelineEdit) => void;
  /** Renames the project, which the window's title names. */
  onRenameProject?: (title: string) => void;
  onResolutionScaleChange?: (scale: number) => void;
  onSave?: () => void;
  onScreenshotFormatChange?: (format: ScreenshotFormat) => void;
  onScreenshotOutputChange?: ScreenshotOutputChange;
  onSelectedScreenshotItemChange?: (itemId: number | null) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
  onSubtitlesChange?: (subtitles: ExportSubtitles) => void;
  onToggleMaximize?: () => void;
  /** A stream's gain over its recorded level, in decibels. */
  onTrackVolumeChange?: (streamIndex: number, decibels: number) => void;
  /** The video, or the sound alone as M4A or WAV. */
  recordingFormat?: RecordingFormat;
  recordingOutput?: RecordingOutputSettings;
  recordingPreviewError?: string | null;
  recordingPreviewLayout?: RecordingPreviewLayout;
  recordingPreviewTracks?: PreparedAudioTrack[];
  recordingTimelineEdit?: RecordingTimelineEdit | null;
  resolutionScalePercent?: number;
  screenshotFormat?: ScreenshotFormat;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedScreenshotItemId?: number | null;
  selectedTrack?: RecordingTrackId | null;
  /** The subtitle file an export writes beside a recording's video. */
  subtitles?: ExportSubtitles;
};
