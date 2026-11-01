// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** What happened over a recording, from its cursor and key files and the
 * picture's readings (`src-tauri/src/editor/recording_activity.rs`), on its
 * own timeline before any edit. */
export type RecordingActivity = {
  clicks: { count: number; ms: number }[];
  picture: { endMs: number; startMs: number; state: "moving" | "new" }[];
  typing: number[];
};

export const getRecordingActivity = (artifactId: number) =>
  invoke<RecordingActivity>("get_recording_activity", { artifactId });
