// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  enabledStreamIndices: number[];
  keyboardEffects: KeyboardEffectSettings;
  recordingOutput: RecordingOutputSettings;
}) => JSON.stringify(settings);
