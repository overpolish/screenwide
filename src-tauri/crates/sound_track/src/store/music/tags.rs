// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a song is described, and the library and the search online narrow
//! songs by: how fast it goes, whether it is sung, how it feels and what
//! kind of music it is. A song found on Openverse comes with these as
//! Jamendo tags it (`MusicTags`); a song added from a file is measured for
//! its tempo and heard for its singing, and has no mood or genre.

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

/// How fast a song goes.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "MusicTempo")]
pub enum Tempo {
  Slow,
  Medium,
  Fast,
}

/// Whether a song is sung.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "MusicVocals")]
pub enum Vocals {
  Instrumental,
  Vocal,
}

/// How a song feels, among those Jamendo tags enough songs with.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "MusicMood")]
pub enum Mood {
  Happy,
  Uplifting,
  Energetic,
  Calm,
  Dreamy,
  Romantic,
  Sad,
  Dark,
  Epic,
}

/// What Openverse says of a song, kept with it in the library. A word this
/// build does not know, as a later Worker may send, is left out rather than
/// failing the whole answer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "MusicTags")]
pub struct MusicTags {
  #[serde(default, deserialize_with = "known")]
  pub tempo: Option<Tempo>,
  #[serde(default, deserialize_with = "known")]
  pub vocals: Option<Vocals>,
  #[serde(default, deserialize_with = "each_known")]
  pub moods: Vec<Mood>,
  /// As Openverse names them, such as `ambient`.
  #[serde(default)]
  pub genres: Vec<String>,
}

fn known<'de, D: Deserializer<'de>, T: serde::de::DeserializeOwned>(
  deserializer: D,
) -> Result<Option<T>, D::Error> {
  Ok(serde_json::from_value(serde_json::Value::deserialize(deserializer)?).ok())
}

fn each_known<'de, D: Deserializer<'de>, T: serde::de::DeserializeOwned>(
  deserializer: D,
) -> Result<Vec<T>, D::Error> {
  Ok(
    Vec::<serde_json::Value>::deserialize(deserializer)?
      .into_iter()
      .filter_map(|value| serde_json::from_value(value).ok())
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn words_this_build_does_not_know_are_left_out() {
    let tags: MusicTags = serde_json::from_str(
      r#"{"tempo":"glacial","vocals":"vocal","moods":["sad","wistful"],"genres":["folk"]}"#,
    )
    .unwrap();
    assert_eq!(
      tags,
      MusicTags {
        tempo: None,
        vocals: Some(Vocals::Vocal),
        moods: vec![Mood::Sad],
        genres: vec!["folk".to_owned()],
      }
    );
  }
}
