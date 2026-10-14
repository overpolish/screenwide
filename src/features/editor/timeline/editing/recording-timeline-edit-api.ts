// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { RecordingTimelineEdit } from "./recording-timeline-edit";

/** An edit waiting to be saved, at its revision. */
type Save = {
  artifactId: number;
  edit: RecordingTimelineEdit;
  revision: number;
};

/** The recordings being saved, each with the latest edit waiting for its
 * save to finish, if one is. */
const saving = new Map<number, Save | null>();

const save = (next: Save) => {
  saving.set(next.artifactId, null);
  invoke<null>("set_recording_timeline_edit", next)
    .catch((cause: unknown) => {
      console.error("Could not persist recording timeline edit", cause);
    })
    .finally(() => {
      const waiting = saving.get(next.artifactId);
      if (waiting) save(waiting);
      else saving.delete(next.artifactId);
    });
};

/**
 * Saves `edit` into its recording's project. A slider dragged changes the
 * edit a step at a time, faster than the project can be written, so the
 * edits that arrive while one is being saved give way to the latest, saved
 * as soon as that one finishes: the project always ends on the last edit,
 * and never writes an earlier one after it.
 */
export const persistRecordingTimelineEdit = (
  artifactId: number,
  revision: number,
  edit: RecordingTimelineEdit,
) => {
  const next = { artifactId, edit, revision };
  if (saving.has(artifactId)) saving.set(artifactId, next);
  else save(next);
};
