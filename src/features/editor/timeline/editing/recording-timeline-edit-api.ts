// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { RecordingTimelineEdit } from "./recording-timeline-edit";

export const persistRecordingTimelineEdit = (
  artifactId: number,
  revision: number,
  edit: RecordingTimelineEdit,
) =>
  invoke<null>("set_recording_timeline_edit", {
    artifactId,
    edit,
    revision,
  });
