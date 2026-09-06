// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingOutputSettings } from "./screenshot-output";

/** Timeline/history restores never reintroduce per-layer background rounding. */
export const recordingOutputForEdit = (
  output: RecordingOutputSettings,
): RecordingOutputSettings => ({
  ...output,
  camera: { ...output.camera, backgroundRadiusPercent: 0 },
  primary: { ...output.primary, backgroundRadiusPercent: 0 },
});
