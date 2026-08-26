// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../recording-timeline-edit";
import {
  RecordingOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  ExportArtifact,
  KeyboardEffectSettings,
  PreparedAudioTrack,
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

import {
  RecordingOutputChange,
  ScreenshotOutputChange,
} from "./export-content";

export type ExportPanelProps = {
  artifact: ExportArtifact | null;
  directory: string | null;
  fileStem: string;
  audioTrackVolumes?: AudioTrackVolume[];
  bakeCamera?: boolean;
  cameraCompression?: number;
  cameraOverlay?: CameraOverlaySettings;
  cameraResolutionScalePercent?: number;
  collapseAudio?: boolean;
  compression?: number;
  cursorEffects?: CursorEffectSettings;
  enabledAudioTrackCount?: number;
  enabledStreamIndices?: number[];
  enabledVideoTracks?: RecordingVideoTrackId[];
  error?: string | null;
  estimatedSizeBytes?: number | null;
  etaSeconds?: number | null;
  isCancelingSave?: boolean;
  isEstimatingSize?: boolean;
  isExportPreparationPending?: boolean;
  isPreparingRecordingAudio?: boolean;
  isPreparingRecordingPreview?: boolean;
  isSaving?: boolean;
  keyboardEffects?: KeyboardEffectSettings;
  onBakeCameraChange?: (bake: boolean) => void;
  onBrowse?: () => void;
  onCameraCompressionChange?: (compression: number) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onCameraResolutionScaleChange?: (scale: number) => void;
  onCancel?: () => void;
  onCancelSave?: () => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCollapseAudioChange?: (collapse: boolean) => void;
  onCompressionChange?: (compression: number) => void;
  onCopy?: () => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onEnabledTracksChange?: (streamIndices: number[]) => void;
  onEnabledVideoTracksChange?: (tracks: RecordingVideoTrackId[]) => void;
  onFileStemChange?: (fileStem: string) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onMinimize?: () => void;
  onRecordingOutputChange?: RecordingOutputChange;
  onRecordingTimelineEditChange?: (edit: RecordingTimelineEdit) => void;
  onResolutionScaleChange?: (scale: number) => void;
  onSave?: () => void;
  onScreenshotBackgroundRadiusChange?: (radiusPercent: number) => void;
  onScreenshotBackgroundRadiusChangeEnd?: () => void;
  onScreenshotOutputChange?: ScreenshotOutputChange;
  onScreenshotRadiusChangeEnd?: () => void;
  onSelectedScreenshotItemChange?: (itemId: number | null) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
  onSelectedTrackVolumeChange?: (decibels: number) => void;
  onToggleMaximize?: () => void;
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
  recordingOutput?: RecordingOutputSettings;
  recordingPreviewError?: string | null;
  recordingPreviewLayout?: RecordingPreviewLayout;
  recordingPreviewTracks?: PreparedAudioTrack[];
  recordingTimelineEdit?: RecordingTimelineEdit | null;
  resolutionScalePercent?: number;
  savePhase?: "camera" | "finalizing" | "recording";
  saveProgress?: number | null;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedScreenshotItemId?: number | null;
  selectedTrack?: RecordingTrackId | null;
};
