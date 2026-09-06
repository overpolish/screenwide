// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DeletedKeyboardShortcutRange,
  KeyboardShortcutPositionRange,
} from "./recording-keyboard-timeline-edit";
import { RecordingTimelinePlaybackRange } from "./recording-timeline-playback";
import { RecordingOutputSettings } from "./screenshot-output";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  CursorEffectSettings,
  KeyboardEffectSettings,
} from "./types";

export const recordingPreviewSettingsKey = (settings: {
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  deletedKeyboardShortcutIds: number[];
  deletedKeyboardShortcutRanges: DeletedKeyboardShortcutRange[];
  enabledStreamIndices: number[];
  keyboardEffects: KeyboardEffectSettings;
  keyboardShortcutPositions: KeyboardShortcutPositionRange[];
  playbackRanges: RecordingTimelinePlaybackRange[];
  recordingOutput: RecordingOutputSettings;
}) => JSON.stringify(settings);
