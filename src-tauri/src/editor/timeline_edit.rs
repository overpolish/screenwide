// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A recording's edit as its project keeps it. The edit itself is
//! `screenwide_timeline_edit`.

use std::path::Path;

use screenwide_timeline_edit::{
  validate, PersistedTimelineEdit, RecordingTimelineEdit, TimelinePlan, FORMAT_VERSION,
};

mod initial_edit;
pub(in crate::editor) use initial_edit::persist_initial_edit;

/// The project's edit and its revision, bound to `artifact_id`. An edit this
/// version cannot read, or one that fails validation, is treated as absent.
pub fn for_project(project: &Path, artifact_id: u64) -> Option<(u64, RecordingTimelineEdit)> {
  let PersistedTimelineEdit {
    mut edit,
    revision,
    version,
  } = screenwide_project::read(project).ok()?.timeline?;
  if version != FORMAT_VERSION || validate(&edit).is_err() {
    return None;
  }
  edit.artifact_id = artifact_id;
  Some((revision, edit))
}

pub fn snapshot_fields(
  project: &Path,
  artifact_id: u64,
) -> (Option<u64>, Option<RecordingTimelineEdit>) {
  for_project(project, artifact_id).map_or((None, None), |(revision, edit)| {
    (Some(revision), Some(edit))
  })
}

/// The plan an export or its estimate follows: the edit the window sent, if
/// it is this recording's, otherwise the one saved in the project.
pub fn export_plan(
  sent: Option<&RecordingTimelineEdit>,
  project: &Path,
  artifact_id: u64,
  duration_ms: u64,
) -> Option<TimelinePlan> {
  match sent.filter(|edit| edit.artifact_id == artifact_id) {
    Some(edit) => TimelinePlan::from_edit(edit, duration_ms),
    None => for_project(project, artifact_id)
      .and_then(|(_, edit)| TimelinePlan::from_edit(&edit, duration_ms)),
  }
}

/// How long the project whose manifest is `project` plays once edited: its
/// recording's `duration_ms` with its cuts and speed changes applied. The
/// recording's own length while it has no edit.
pub fn edited_duration_ms(project: &Path, duration_ms: u64) -> u64 {
  export_plan(None, project, 0, duration_ms).map_or(duration_ms, |plan| plan.duration_ms())
}

pub fn persist(
  project: &Path,
  artifact_id: u64,
  revision: u64,
  edit: RecordingTimelineEdit,
) -> Result<(), String> {
  if edit.artifact_id != artifact_id {
    return Err("That timeline belongs to another recording".to_owned());
  }
  validate(&edit)?;
  screenwide_project::update(project, |manifest| {
    if manifest
      .timeline
      .as_ref()
      .is_some_and(|current| current.revision >= revision)
    {
      return false;
    }
    manifest.timeline = Some(PersistedTimelineEdit {
      edit,
      revision,
      version: FORMAT_VERSION,
    });
    true
  })
}

#[cfg(test)]
mod tests;
