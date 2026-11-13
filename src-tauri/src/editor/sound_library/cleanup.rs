// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Takes out of a project the sounds its clips no longer play.
//!
//! While the editor has a project open, every sound added is copied in and
//! none is taken out, so undo can bring back a clip that was deleted. Once
//! the project is opened or let go there is no undo, so what no clip plays
//! goes, with the singing separated from it. The library keeps them all.

use std::path::Path;

use screenwide_sound_track::store;

/// Tidies the project whose manifest is `project`. An edit this build cannot
/// read leaves every sound where it is.
pub(crate) fn clean(project: &Path) {
  let (Some(folder), Ok(manifest)) = (
    super::project_sounds(project),
    screenwide_project::read(project),
  ) else {
    return;
  };
  let Some(duration_ms) = manifest
    .recorded()
    .ok()
    .and_then(|recorded| recorded.duration_ms)
  else {
    return;
  };
  let clips = match manifest.timeline {
    Some(persisted) => match persisted.into_current(duration_ms) {
      Some(edit) => edit.audio_clips,
      None => return,
    },
    None => Vec::new(),
  };
  store::prune(&folder, clips.iter().map(|clip| clip.sound.as_str()));
}
