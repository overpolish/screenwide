// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The captions' word menu, opened by a right press on a word of the caption
//! showing on the picture, and what its picks ask of the picture: opening
//! the caption for typing after a word, or, in the caption being typed into,
//! taking a word out or putting back what was heard.

use super::caption_text::CaptionEditSpan;
use super::*;
use screenwide_text::captions::{typed_caption_words, CaptionWord};
use screenwide_text::TextAccent;

/// A word of the caption being typed into: its bytes in the text, and the
/// run of the words typed for the same words said, the word among them, by
/// where it starts and as it reads, which the word's respelling is read
/// against.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptionTypedWord {
  start: usize,
  end: usize,
  run_start: usize,
  run_text: String,
}

/// A right press on a word of the caption showing: the word, by the stretch
/// of the source it was said in, the word before it, which the gap a break
/// sits in is between, null where no break can sit there, whether the word
/// starts its caption, and, where the captions set a key word apart,
/// whether it is one. Where the caption is being typed into, the word is one
/// as typed, and `typed` where it is in the text. `x` and `y` are where the
/// menu opens, as the context menu's are.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptionWordMenu {
  session_id: u64,
  word: CaptionEditSpan,
  before: Option<CaptionEditSpan>,
  starts_caption: bool,
  is_key_word: Option<bool>,
  typed: Option<CaptionTypedWord>,
  x: f64,
  y: f64,
}

fn span([start_ms, end_ms]: [f64; 2]) -> CaptionEditSpan {
  CaptionEditSpan { start_ms, end_ms }
}

fn word_span(word: &CaptionWord) -> [f64; 2] {
  [word.source_start_ms, word.source_end_ms]
}

/// Opens the caption showing for typing, the caret after the word said at
/// `word_ms` in the source: the word menu's Edit.
#[tauri::command]
pub async fn edit_recording_caption(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  word_ms: f64,
) -> Result<(), String> {
  let mut manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview is unavailable")?;
  manager.require_session(session_id)?;
  manager.begin_caption_session(Some(word_ms));
  Ok(())
}

/// Types `text` over bytes `start` to `end` of the caption being typed into,
/// as typing would: the word menu's Restore spelling and, with no text,
/// Hide while the caption is typed into. A word taken out takes a space
/// beside it along.
#[tauri::command]
pub async fn retype_caption_text(
  state: tauri::State<'_, RecordingPreviewPlayerState>,
  session_id: u64,
  start: usize,
  end: usize,
  text: String,
) -> Result<(), String> {
  let manager = state
    .0
    .lock()
    .map_err(|_| "The recording preview is unavailable")?;
  manager.require_session(session_id)?;
  let Some(typed) = manager
    .annotation
    .caption
    .as_ref()
    .map(|session| session.editing.text.as_str())
  else {
    return Ok(());
  };
  if start >= end || typed.get(start..end).is_none() {
    return Ok(());
  }
  let (start, end) = if !text.is_empty() {
    (start, end)
  } else if typed[end..].starts_with(' ') {
    (start, end + 1)
  } else if typed[..start].ends_with(' ') {
    (start - 1, end)
  } else {
    (start, end)
  };
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(surface) = manager
    .sources
    .as_ref()
    .and_then(|sources| sources.preview_surface.as_ref())
  {
    surface.replace_caption_text(start, end, &text);
  }
  Ok(())
}

impl PreviewPlayerManager {
  /// The word menu for a right press at `x`, `y` on the caption showing at
  /// the playhead. `at` is the word pressed, counted across its lines, or,
  /// while the caption is typed into, the byte of its text the press landed
  /// at.
  pub(in crate::editor::recording_preview_player) fn caption_word_menu(
    &self,
    at: usize,
    (x, y): (f64, f64),
  ) -> Option<CaptionWordMenu> {
    let (track, moment) = self.sources.as_ref()?.captions_at(self.position_ms, 0.0)?;
    let keyed = track.recording.style.accent == TextAccent::Word;
    let menu = |word: [f64; 2], before: Option<[f64; 2]>, starts_caption, is_key_word, typed| {
      Some(CaptionWordMenu {
        session_id: self.session_id?,
        word: span(word),
        before: before.map(span),
        starts_caption,
        is_key_word: keyed.then_some(is_key_word),
        typed,
        x,
        y,
      })
    };
    if let Some(session) = self.annotation.caption.as_ref() {
      let words = typed_caption_words(&session.caption, &session.editing.text);
      let index = words
        .iter()
        .position(|word| word.bytes.start <= at && at <= word.bytes.end)?;
      let word = &words[index];
      // A word typed beside another that stands for the same words has no
      // gap between them for a break.
      let before = match index.checked_sub(1) {
        Some(before) => Some(words[before].span).filter(|before| *before != word.span),
        None => track
          .captions
          .iter()
          .flat_map(|caption| caption.lines.iter().flatten())
          .map(word_span)
          .filter(|said| said[1] <= word.span[0])
          .max_by(|a, b| a[1].total_cmp(&b[1])),
      };
      let is_key_word = session
        .editing
        .emphasis
        .iter()
        .any(|apart| apart.start < word.bytes.end && word.bytes.start < apart.end);
      // The words typed for the same words said run on from one another.
      let first = words[..index]
        .iter()
        .rposition(|other| other.span != word.span)
        .map_or(0, |other| other + 1);
      let last = words[index..]
        .iter()
        .position(|other| other.span != word.span)
        .map_or(words.len(), |other| index + other)
        - 1;
      let run = words[first].bytes.start..words[last].bytes.end;
      let typed = CaptionTypedWord {
        start: word.bytes.start,
        end: word.bytes.end,
        run_start: run.start,
        run_text: session.editing.text[run].to_owned(),
      };
      return menu(word.span, before, index == 0, is_key_word, Some(typed));
    }
    let showing = track.showing_at(moment.at_ms)?;
    let position = track
      .captions
      .iter()
      .position(|caption| caption.id == showing.id)?;
    let words: Vec<_> = showing.lines.iter().flatten().collect();
    let word = *words.get(at)?;
    let before = match at.checked_sub(1) {
      Some(before) => words.get(before).copied(),
      None => position
        .checked_sub(1)
        .and_then(|previous| track.captions[previous].lines.iter().flatten().last()),
    };
    menu(
      word_span(word),
      before.map(word_span),
      at == 0,
      word.emphasis,
      None,
    )
  }
}
