// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The credits an export owes for the CC BY songs, sound effects and
//! pictures its project uses, found on Openverse: a text file beside the
//! exported file, named after it, to paste wherever the export is shared.
//! None is written where nothing is owed. What a project uses is read from
//! its manifest, as saved after every change: the sounds its clips play and
//! the pictures its images show, each credit from the library or else from
//! the copy the project carries.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The sound every audio clip in `value` plays, wherever in it.
fn sound_names(value: &Value, names: &mut BTreeSet<String>) {
  match value {
    Value::Object(fields) => {
      if let Some(Value::Array(clips)) = fields.get("audioClips") {
        names.extend(
          clips
            .iter()
            .filter_map(|clip| clip.get("sound")?.as_str().map(str::to_owned)),
        );
      }
      fields.values().for_each(|field| sound_names(field, names));
    }
    Value::Array(values) => values.iter().for_each(|value| sound_names(value, names)),
    _ => {}
  }
}

/// Where the credits for `exported` go: beside it, named after it.
fn credits_path(exported: &Path) -> Option<PathBuf> {
  let stem = exported.file_stem()?.to_str()?;
  Some(exported.with_file_name(format!("{stem}.credits.txt")))
}

/// Writes the credits the project whose manifest is `project` owes beside
/// `exported`, and says where; none where nothing is owed or they could not
/// be written, which leaves the export as it is.
pub(super) fn write_beside(project: &Path, exported: &Path) -> Option<PathBuf> {
  let manifest: Value = serde_json::from_slice(&std::fs::read(project).ok()?).ok()?;
  let mut sounds = BTreeSet::new();
  sound_names(&manifest, &mut sounds);
  let mut pictures = BTreeSet::new();
  crate::editor::project_look::stored_names(&manifest, &mut pictures);
  let carried_sounds = crate::editor::sound_library::project_sounds(project);
  let carried_pictures = crate::editor::project_look::pictures_folder(project).ok();
  let credits: Vec<_> = sounds
    .iter()
    .filter_map(|name| {
      screenwide_sound_track::store::resolve(carried_sounds.as_deref(), name)?
        .1
        .credit
    })
    .chain(
      pictures
        .iter()
        .filter_map(|name| screenwide_images::store::credit_in(carried_pictures.as_deref(), name)),
    )
    .collect();
  let text = screenwide_credits::credits_text(&credits)?;
  let path = credits_path(exported)?;
  match std::fs::write(&path, text) {
    Ok(()) => Some(path),
    Err(error) => {
      eprintln!("Could not write the credits beside the export: {error}");
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_clips_sound_is_found_wherever_the_edit_keeps_it() {
    let manifest = serde_json::json!({
      "edit": { "audioClips": [{ "sound": "a.mp3" }, { "sound": "b.flac" }] },
      "other": [{ "audioClips": [{ "sound": "a.mp3" }] }],
    });
    let mut names = BTreeSet::new();
    sound_names(&manifest, &mut names);
    assert_eq!(names.into_iter().collect::<Vec<_>>(), ["a.mp3", "b.flac"]);
  }

  #[test]
  fn the_credits_are_named_after_the_export_beside_it() {
    assert_eq!(
      credits_path(Path::new("/out/Demo video.mp4")),
      Some(PathBuf::from("/out/Demo video.credits.txt"))
    );
  }
}
