// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The word list: names and terms speech is written with, typed in Settings
//! or added by voice while dictating. Dictation writes a listed word where
//! the audio fits it better than what was heard; recording transcripts and
//! voice notes give it to Whisper as its prompt. Earlier words come first
//! where an engine takes only so many.

use std::sync::{PoisonError, RwLock};

use tauri::{AppHandle, Manager};

const FILE: &str = "words.json";

/// Longer than any name or term: the dictation engine skips terms past 128
/// bytes.
const LONGEST: usize = 64;

static WORDS: RwLock<Vec<String>> = RwLock::new(Vec::new());

pub(crate) fn current() -> Vec<String> {
  WORDS.read().unwrap_or_else(PoisonError::into_inner).clone()
}

fn path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join(FILE))
    .map_err(|error| error.to_string())
}

pub fn initialize(app: &AppHandle) {
  let stored = path(app)
    .ok()
    .and_then(|path| std::fs::read(path).ok())
    .and_then(|bytes| serde_json::from_slice::<Vec<String>>(&bytes).ok());
  if let Some(words) = stored {
    *WORDS.write().unwrap_or_else(PoisonError::into_inner) = tidy(words);
  }
}

/// Keeps `words` as the list, tidied, and tells every window.
pub(crate) fn set(app: &AppHandle, words: Vec<String>) -> Result<(), String> {
  let words = tidy(words);
  let path = path(app)?;
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(&words).map_err(|error| error.to_string())?;
  std::fs::write(path, bytes).map_err(|error| error.to_string())?;
  *WORDS.write().unwrap_or_else(PoisonError::into_inner) = words;
  super::models::changed(app);
  Ok(())
}

/// Adds `word` at the end, unless it is listed already. Whether it was added.
pub(crate) fn add(app: &AppHandle, word: &str) -> Result<bool, String> {
  let mut words = current();
  if words.iter().any(|listed| same(listed, word)) {
    return Ok(false);
  }
  words.push(word.to_owned());
  set(app, words).map(|()| true)
}

/// Takes `word` off the list. Whether it was listed.
pub(crate) fn remove(app: &AppHandle, word: &str) -> Result<bool, String> {
  let mut words = current();
  let before = words.len();
  words.retain(|listed| !same(listed, word));
  if words.len() == before {
    return Ok(false);
  }
  set(app, words).map(|()| true)
}

/// A word listed twice in different cases is one word.
fn same(a: &str, b: &str) -> bool {
  a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Each word trimmed, of a sensible length, and listed once, first place
/// kept.
fn tidy(words: Vec<String>) -> Vec<String> {
  let mut kept: Vec<String> = Vec::with_capacity(words.len());
  for word in words {
    let word = word.split_whitespace().collect::<Vec<_>>().join(" ");
    if word.is_empty()
      || word.chars().count() > LONGEST
      || kept.iter().any(|listed| same(listed, &word))
    {
      continue;
    }
    kept.push(word);
  }
  kept
}

#[cfg(test)]
mod tests {
  use super::tidy;

  #[test]
  fn keeps_each_word_once_in_its_first_spelling() {
    let words = [
      "  Overpolish ",
      "Screenwide",
      "overpolish",
      "",
      "Open  Code",
    ]
    .map(str::to_owned);
    assert_eq!(
      tidy(words.to_vec()),
      ["Overpolish", "Screenwide", "Open Code"]
    );
  }
}
