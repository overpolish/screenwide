// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Openverse, searched for songs, sound effects and pictures to add to the
//! library. The app never asks Openverse itself: it asks Screenwide's
//! Worker (`workers/openverse`), which holds the app's registration and only
//! ever answers with what may be used with no more than a credit. This
//! crate is what is asked and what comes back; the app sends the request
//! (`src/editor/openverse.rs`), as it does the licence server's.

mod install;
mod music;

pub use install::{install_id, INSTALL_HEADER};
pub use music::MusicFilters;
use screenwide_credits::Credit;
use screenwide_sound_track::store::music::MusicTags;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Where the app asks: Screenwide's Worker.
pub const WORKER_URL: &str = "https://openverse.getscreenwide.com";

/// What the library searches for: songs, sound effects or pictures.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "OpenverseKind")]
pub enum Kind {
  Music,
  Effect,
  Image,
}

impl Kind {
  fn as_str(self) -> &'static str {
    match self {
      Self::Music => "music",
      Self::Effect => "effect",
      Self::Image => "image",
    }
  }
}

/// One work found, as the Worker answers with it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "OpenverseFound")]
pub struct Found {
  pub id: String,
  pub title: String,
  pub creator: Option<String>,
  pub creator_url: Option<String>,
  /// How long it plays, in ms; none for a picture.
  pub duration_ms: Option<f64>,
  pub filetype: Option<String>,
  /// What it is as music, as Jamendo tags it; empty for others.
  #[serde(default)]
  pub tags: MusicTags,
  pub width: Option<f64>,
  pub height: Option<f64>,
  pub license: String,
  pub license_version: Option<String>,
  pub license_url: Option<String>,
  /// Its page where it is published.
  pub landing_url: Option<String>,
  /// A small picture of it, for a picture.
  pub thumbnail: Option<String>,
  /// The file itself.
  pub url: String,
  /// The credit line its licence asks for.
  pub attribution: String,
}

impl Found {
  /// Who made it and under which licence, kept with it in the library.
  pub fn credit(&self) -> Credit {
    Credit {
      title: self.title.clone(),
      creator: self.creator.clone(),
      creator_url: self.creator_url.clone(),
      license: self.license.clone(),
      license_version: self.license_version.clone(),
      license_url: self.license_url.clone(),
      source_url: self.landing_url.clone(),
      attribution: self.attribution.clone(),
    }
  }
}

/// A page of what a search found, and how many pages it has.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "OpenversePage")]
pub struct Page {
  pub found: Vec<Found>,
  pub page_count: u32,
}

/// Why a search found nothing to show.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "OpenverseFailure")]
pub enum Failure {
  /// Too many searches this minute, from here or from everyone.
  Busy,
  /// This installation's searches for the day are used up, until `until`,
  /// in ms since the epoch.
  UsedUp { until: f64 },
  /// Openverse, or the way to it, could not answer.
  Unavailable,
}

/// The Worker's answer: `status` and its body.
pub fn page_from(status: u16, body: &[u8]) -> Result<Page, Failure> {
  #[derive(Deserialize)]
  struct Refusal {
    error: String,
    until: Option<f64>,
  }
  match status {
    200 => serde_json::from_slice(body).map_err(|_| Failure::Unavailable),
    429 => Err(match serde_json::from_slice::<Refusal>(body) {
      Ok(Refusal {
        error,
        until: Some(until),
      }) if error == "used_up" => Failure::UsedUp { until },
      _ => Failure::Busy,
    }),
    _ => Err(Failure::Unavailable),
  }
}

/// Why a work found could not be added.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorkFailure {
  /// Gone, or no longer under a licence the app may use.
  Withdrawn,
  /// Too many requests for now.
  Busy,
  /// Openverse, or the way to it, could not answer.
  Unavailable,
}

/// The work as the Worker looked it up again: `status` and its body.
pub fn work_from(status: u16, body: &[u8]) -> Result<Found, WorkFailure> {
  #[derive(Deserialize)]
  struct Answer {
    found: Option<Found>,
  }
  match status {
    200 => serde_json::from_slice::<Answer>(body)
      .map_err(|_| WorkFailure::Unavailable)?
      .found
      .ok_or(WorkFailure::Withdrawn),
    429 => Err(WorkFailure::Busy),
    _ => Err(WorkFailure::Unavailable),
  }
}

/// `text` as it goes in a query string.
fn encoded(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  for byte in text.bytes() {
    if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
      out.push(char::from(byte));
    } else {
      out.push_str(&format!("%{byte:02X}"));
    }
  }
  out
}

/// The address the Worker at `base` is asked at for `page` of what `kind`
/// finds for `query`, music narrowed by `filters`.
pub fn search_url(
  base: &str,
  kind: Kind,
  query: &str,
  filters: &MusicFilters,
  page: u32,
) -> String {
  let mut url = format!(
    "{base}/search?kind={}&q={}&page={page}",
    kind.as_str(),
    encoded(query.trim())
  );
  if kind == Kind::Music {
    for (name, value) in filters.params() {
      url.push_str(&format!("&{name}={value}"));
    }
  }
  url
}

/// The address the Worker at `base` is asked at for the work `id` of `kind`
/// as it stands now.
pub fn work_url(base: &str, kind: Kind, id: &str) -> String {
  format!("{base}/work?kind={}&id={}", kind.as_str(), encoded(id))
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_sound_track::store::music::{Mood, Tempo};

  #[test]
  fn a_search_is_asked_with_its_words_encoded() {
    assert_eq!(
      search_url(
        WORKER_URL,
        Kind::Effect,
        " rain & café ",
        &MusicFilters::default(),
        2
      ),
      "https://openverse.getscreenwide.com/search?kind=effect&q=rain%20%26%20caf%C3%A9&page=2"
    );
  }

  #[test]
  fn music_is_asked_with_the_filters_set() {
    let filters = MusicFilters {
      tempo: Some(Tempo::Slow),
      vocals: None,
      mood: Some(Mood::Calm),
    };
    assert_eq!(
      search_url(WORKER_URL, Kind::Music, "", &filters, 1),
      "https://openverse.getscreenwide.com/search?kind=music&q=&page=1&mood=calm&tempo=slow"
    );
    assert_eq!(
      search_url(WORKER_URL, Kind::Effect, "door", &filters, 1),
      "https://openverse.getscreenwide.com/search?kind=effect&q=door&page=1"
    );
  }

  #[test]
  fn the_workers_answer_is_a_page_or_why_not() {
    let body = br#"{"found":[{"id":"a1","title":"Rain","creator":"Ana","creatorUrl":null,"durationMs":4200,"filetype":"mp3","width":null,"height":null,"license":"by","licenseVersion":"4.0","licenseUrl":null,"landingUrl":"https://freesound.org/s/1","thumbnail":null,"url":"https://cdn.test/rain.mp3","attribution":"\"Rain\" by Ana"}],"pageCount":3}"#;
    let page = page_from(200, body).unwrap();
    assert_eq!(page.page_count, 3);
    assert!(page.found[0].credit().is_owed());
    assert_eq!(page_from(429, br#"{"error":"busy"}"#), Err(Failure::Busy));
    assert_eq!(
      page_from(429, br#"{"error":"used_up","until":1763164800000}"#),
      Err(Failure::UsedUp {
        until: 1_763_164_800_000.0
      })
    );
    assert_eq!(page_from(502, b"{}"), Err(Failure::Unavailable));
  }

  #[test]
  fn a_work_looked_up_again_is_added_only_while_it_is_offered() {
    assert_eq!(
      work_url(WORKER_URL, Kind::Music, "4bc43a04"),
      "https://openverse.getscreenwide.com/work?kind=music&id=4bc43a04"
    );
    let body = br#"{"found":{"id":"a1","title":"Rain","creator":null,"creatorUrl":null,"durationMs":4200,"filetype":"mp3","width":null,"height":null,"license":"cc0","licenseVersion":null,"licenseUrl":null,"landingUrl":null,"thumbnail":null,"url":"https://cdn.test/rain.mp3","attribution":""}}"#;
    assert_eq!(work_from(200, body).unwrap().license, "cc0");
    assert_eq!(
      work_from(200, br#"{"found":null}"#),
      Err(WorkFailure::Withdrawn)
    );
    assert_eq!(work_from(429, b"{}"), Err(WorkFailure::Busy));
  }
}
