// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../recording-timeline-edit";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
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

/** What the screenshot workspace is shown, and what it reports back. */
export type ScreenshotSectionProps = {
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>;
  isSaving?: boolean;
  isSheetOpen?: boolean;
  onBackgroundRadiusChange?: (radiusPercent: number) => void;
  onBackgroundRadiusChangeEnd?: () => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  onRadiusChangeEnd?: () => void;
  onSelectedItemChange?: (itemId: number | null) => void;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedItemId?: number | null;
};

/** The recording workspace's own contract, sibling to the screenshot one. */
export type RecordingSectionProps = {
  artifact: Extract<EditorArtifact, { kind: "recording" }>;
  audioTrackVolumes?: AudioTrackVolume[];
  bakeCamera?: boolean;
  cameraOverlay?: CameraOverlaySettings;
  cameraResolutionScalePercent?: number;
  cursorEffects?: CursorEffectSettings;
  enabledStreamIndices?: number[];
  enabledVideoTracks?: RecordingVideoTrackId[];
  hasCursorData?: boolean;
  hasKeyboardData?: boolean;
  isPreparingRecordingAudio?: boolean;
  isPreparingRecordingPreview?: boolean;
  isSaving?: boolean;
  isSheetOpen?: boolean;
  keyboardEffects?: KeyboardEffectSettings;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onEnabledTracksChange?: (streamIndices: number[]) => void;
  onEnabledVideoTracksChange?: (tracks: RecordingVideoTrackId[]) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onRecordingOutputChange?: (
    trackId: RecordingVideoTrackId,
    settings: RecordingOutputSettings[RecordingVideoTrackId],
  ) => void;
  onRecordingTimelineEditChange?: (edit: RecordingTimelineEdit) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
  recordingOutput?: RecordingOutputSettings;
  recordingPreviewError?: string | null;
  recordingPreviewLayout?: RecordingPreviewLayout;
  recordingPreviewTracks?: PreparedAudioTrack[];
  recordingTimelineEdit?: RecordingTimelineEdit | null;
  resolutionScalePercent?: number;
  selectedTrack?: RecordingTrackId | null;
};
