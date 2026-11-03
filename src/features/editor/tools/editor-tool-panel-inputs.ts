// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CameraOutput } from "../export/camera-output";
import type { EditorExportSnapshot } from "../export/editor-export";
import type {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot/screenshot-output";
import type { RecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import type {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  EditorArtifact,
  EditorKind,
  KeyboardEffectSettings,
  RecordingVideoTrackId,
} from "../types";
import type { ToolPanelExportPatch } from "./tool-panel-export";
import type { TrackInclusion } from "./tracks/editor-tracks";

/** What the editor hands `useEditorToolPanels`: the settings it owns, and
 * the handlers that change them. */
export type EditorToolPanelInputs = {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraOutput: CameraOutput | null;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  enabledVideoTracks: RecordingVideoTrackId[];
  /** What the Export island shows, and where its asks go; the general
   * setting it shows is added where the general settings are read. */
  exportIsland: {
    onPatch: (patch: ToolPanelExportPatch) => void;
    snapshot: EditorExportSnapshot | null;
  };
  isLocked: boolean;
  keyboardEffects: KeyboardEffectSettings;
  recordingOutput: RecordingOutputSettings | null | undefined;
  recordingTimelineEdit: RecordingTimelineEdit | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  trackInclusion: TrackInclusion;
  workspace: EditorKind;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
  onRecordingTimelineEditChange?: (edit: RecordingTimelineEdit) => void;
  onScreenshotOutputChange?: (
    next: ScreenshotOutputSettings,
    itemId: number,
  ) => void;
  /** Plays the audio stream `streamIndex` this much louder or quieter than
   * it was recorded, in decibels. */
  onTrackVolumeChange?: (streamIndex: number, decibels: number) => void;
};
