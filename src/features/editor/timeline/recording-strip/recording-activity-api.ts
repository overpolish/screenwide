// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { RecordingActivity } from "../../../../bindings/RecordingActivity";

export type { RecordingActivity };

/** What happened over the recording open in the editor, from its cursor and
 * key files and the picture's readings
 * (`src-tauri/src/editor/recording_moments/activity.rs`), on its own
 * timeline before any edit. */
export const getRecordingActivity = (artifactId: number) =>
  invoke<RecordingActivity>("get_recording_activity", { artifactId });
