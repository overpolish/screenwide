// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  KeyboardEffectSettings,
  PreparedAudioTrack,
  RecordingPreviewLayout,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../../types";

import { NativeRecordingPreview } from "./native-recording-preview";

export type ScrubPreviewProps = {
  artifactId: number;
  durationMs: number;
  previewSourceDimensions: Partial<
    Record<RecordingVideoTrackId, { height: number; width: number }>
  >;
  audioError?: string | null;
  audioTrackVolumes?: AudioTrackVolume[];
  audioTracks?: PreparedAudioTrack[];
  bakeCamera?: boolean;
  cameraOverlay?: CameraOverlaySettings;
  cursorEffects?: CursorEffectSettings;
  enabledStreamIndices?: number[];
  enabledVideoTracks?: RecordingVideoTrackId[];
  /** Whether annotations can be drawn, which takes a picture to draw them
   * on. */
  hasAnnotations?: boolean;
  hasCursorData?: boolean;
  hasKeyboardData?: boolean;
  /** Whether the recording has a microphone track to transcribe. */
  hasMicrophone?: boolean;
  isPreparingAudio?: boolean;
  isPreparingPreview?: boolean;
  isSaving?: boolean;
  isSheetOpen?: boolean;
  keyboardEffects?: KeyboardEffectSettings;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onEnabledTracksChange?: (streamIndices: number[]) => void;
  onEnabledVideoTracksChange?: (tracks: RecordingVideoTrackId[]) => void;
  onRecordingOutputChange?: (
    trackId: RecordingVideoTrackId,
    settings: RecordingOutputSettings[RecordingVideoTrackId],
  ) => void;
  onRecordingTimelineEditChange?: (edit: RecordingTimelineEdit) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
  previewLayout?: RecordingPreviewLayout;
  previewOutputDimensions?: Partial<
    Record<RecordingVideoTrackId, { height: number; width: number }>
  >;
  recordingOutput?: RecordingOutputSettings;
  recordingTimelineEdit?: RecordingTimelineEdit | null;
  /** Captured pixels per logical point of the screen, which a screen
   * annotation's size is measured in. */
  screenCaptureScale?: number;
  selectedTrack?: RecordingTrackId | null;
};

/** The native Rust player is the sole recording-preview architecture. */
export function ScrubPreview(props: ScrubPreviewProps) {
  return <NativeRecordingPreview {...props} />;
}
