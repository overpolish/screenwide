// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a search for music is narrowed by, in the words the library
//! describes songs with (`screenwide_sound_track::store::music`). The Worker
//! turns each choice into the tag Jamendo gives its tracks for it
//! (`workers/openverse/src/music.ts`).

use screenwide_sound_track::store::music::{Mood, Tempo, Vocals};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a search for music is narrowed by; each left unset takes any.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MusicFilters {
  #[ts(optional)]
  pub tempo: Option<Tempo>,
  #[ts(optional)]
  pub vocals: Option<Vocals>,
  #[ts(optional)]
  pub mood: Option<Mood>,
}

impl MusicFilters {
  /// Each filter set, as the Worker's query names it and its choice: their
  /// names as serde writes them, so they are spelled once.
  pub(crate) fn params(&self) -> Vec<(String, String)> {
    let Ok(serde_json::Value::Object(set)) = serde_json::to_value(self) else {
      return Vec::new();
    };
    set
      .into_iter()
      .filter_map(|(name, value)| Some((name, value.as_str()?.to_owned())))
      .collect()
  }
}
