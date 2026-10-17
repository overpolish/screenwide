// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into the caption showing, on the picture.
//!
//! The caption stands still with every word showing, as the transcript spells
//! them, and the typing goes through the same native typing a text box's
//! does. Every change is drawn at once and reaches React as it is typed: the
//! words spelled another way as corrections of the transcript, and the words
//! taken out whole as hidden, the same edits the Captions panel makes, so the
//! panels follow the typing. Each change is the whole of what the typing has
//! changed so far, against the caption as it was opened.

use super::*;
use crate::editor::preview_platform::AnnotationTextPhase;
use screenwide_text::captions::{
  caption_changes, editing_style, typed_emphasis, Caption, CaptionEditing, CaptionTrack,
};
use screenwide_text::emphasis::follow_edit;
use screenwide_text::TextStyle;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counts the typings of captions on the picture, so each one's changes are
/// told apart from another's.
static TYPINGS: AtomicU64 = AtomicU64::new(0);

pub(in crate::editor::recording_preview_player) struct CaptionSession {
  /// Which typing this is, as its changes name it.
  typing: u64,
  /// The caption as it was opened, which the typing is compared against.
  pub(super) caption: Caption,
  pub(super) editing: CaptionEditing,
  /// The style the typing is set in, as the native typing last had it.
  look: TextStyle,
  revision: u64,
}

/// A stretch of the source, in milliseconds.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptionEditSpan {
  pub(super) start_ms: f64,
  pub(super) end_ms: f64,
}

/// A correction of the transcript typing on the picture made.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptionEditCorrection {
  start_ms: f64,
  end_ms: f64,
  text: String,
}

/// What React is to apply for a caption typed into on the picture: the
/// transcript's corrections and the words to hide that the typing makes so
/// far, which stand in for those of the change before; how long the
/// recording is, which the captions' word choices are placed by; and whether
/// the typing goes on or has ended. `typing` names the typing they are of.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptionEditCommit {
  session_id: u64,
  typing: u64,
  phase: &'static str,
  corrections: Vec<CaptionEditCorrection>,
  hidden: Vec<CaptionEditSpan>,
  duration_ms: f64,
}

impl PreviewPlayerManager {
  /// Draws the captions with `editing` typed in the place of the caption it
  /// was made from, or as they are with none.
  fn show_caption_editing(&self, editing: Option<CaptionEditing>) {
    let Some(sources) = self.sources.as_ref() else {
      return;
    };
    let mut shown = sources
      .captions
      .write()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(track) = shown.as_mut() {
      if track.editing != editing {
        Arc::make_mut(track).editing = editing;
      }
    }
  }

  /// The caption being typed into, if any, as `track`, the captions made
  /// again meanwhile, keeps showing it: its key words as they set them
  /// apart now, which the native typing lays its words out by too.
  pub(in crate::editor::recording_preview_player) fn caption_editing_in(
    &mut self,
    track: &CaptionTrack,
  ) -> Option<CaptionEditing> {
    let session = self.annotation.caption.as_mut()?;
    let emphasis = typed_emphasis(&session.caption, &session.editing.text, &track.captions);
    let look = editing_style(&track.recording.style);
    if emphasis != session.editing.emphasis || look != session.look {
      session.editing.emphasis = emphasis;
      session.look = look;
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      if let Some(surface) = self
        .sources
        .as_ref()
        .and_then(|sources| sources.preview_surface.as_ref())
      {
        surface.set_annotation_text_words(
          &session.editing.text,
          &session.look,
          &session.editing.emphasis,
          true,
        );
      }
    }
    Some(session.editing.clone())
  }

  /// Opens the caption showing at the playhead for typing, the caret at the
  /// end of the word said at `word_ms` in the source, where given, or where
  /// the press that opened it landed. Answers whether it opened.
  pub(in crate::editor::recording_preview_player) fn begin_caption_session(
    &mut self,
    word_ms: Option<f64>,
  ) -> bool {
    if self.session_id.is_none()
      || self.is_playing
      || self.annotation.gesture.is_some()
      || self.annotation.text.is_some()
      || self.annotation.caption.is_some()
    {
      return false;
    }
    let Some(sources) = self.sources.as_ref() else {
      return false;
    };
    let Some((track, moment)) = sources.captions_at(self.position_ms, 0.0) else {
      return false;
    };
    let Some(caption) = track.showing_at(moment.at_ms).cloned() else {
      return false;
    };
    let editing = CaptionEditing::of(&caption);
    let caret = word_ms.and_then(|at| caret_after_word(&caption, &editing.text, at));
    let look = editing_style(&track.recording.style);
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Some(surface) = sources.preview_surface.as_ref() {
      surface.begin_caption_text(&editing.text, &look, &editing.emphasis, caret);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = caret;
    self.annotation.caption = Some(CaptionSession {
      typing: TYPINGS.fetch_add(1, Ordering::Relaxed),
      caption,
      editing: editing.clone(),
      look,
      revision: 0,
    });
    self.show_caption_editing(Some(editing));
    let _ = self.restart(PlaybackMode::InteractiveStill);
    true
  }

  /// One report from the caption being typed into. A change is drawn and
  /// answers what React is to apply so far; the end answers what it is to
  /// keep.
  pub(super) fn caption_text(
    &mut self,
    phase: AnnotationTextPhase,
    text: &str,
    revision: u64,
  ) -> Option<CaptionEditCommit> {
    match phase {
      AnnotationTextPhase::Open => {
        self.begin_caption_session(None);
        None
      }
      AnnotationTextPhase::Change => {
        let session = self.annotation.caption.as_mut()?;
        if revision <= session.revision {
          return None;
        }
        session.revision = revision;
        take_text(&mut session.editing, text);
        let editing = session.editing.clone();
        self.show_caption_editing(Some(editing));
        let _ = self.restart(PlaybackMode::InteractiveStill);
        self.caption_commit("change")
      }
      AnnotationTextPhase::End => {
        if let Some(session) = self
          .annotation
          .caption
          .as_mut()
          .filter(|session| revision > session.revision)
        {
          take_text(&mut session.editing, text);
        }
        let commit = self.caption_commit("end");
        self.annotation.caption = None;
        self.show_caption_editing(None);
        let _ = self.restart(PlaybackMode::InteractiveStill);
        commit
      }
    }
  }

  /// What the caption being typed into has changed so far, for `phase`.
  fn caption_commit(&self, phase: &'static str) -> Option<CaptionEditCommit> {
    let session = self.annotation.caption.as_ref()?;
    let changes = caption_changes(&session.caption, &session.editing.text);
    Some(CaptionEditCommit {
      session_id: self.session_id?,
      typing: session.typing,
      phase,
      corrections: changes
        .corrections
        .into_iter()
        .map(|correction| CaptionEditCorrection {
          start_ms: correction.start_ms,
          end_ms: correction.end_ms,
          text: correction.text,
        })
        .collect(),
      hidden: changes
        .hidden
        .into_iter()
        .map(|[start_ms, end_ms]| CaptionEditSpan { start_ms, end_ms })
        .collect(),
      duration_ms: self.sources.as_ref()?.duration_ms as f64,
    })
  }
}

/// `editing` holding `text` as typed, its key words carried along.
fn take_text(editing: &mut CaptionEditing, text: &str) {
  let text = text.replace("\r\n", "\n").replace('\r', "\n");
  editing.emphasis = follow_edit(&editing.emphasis, &editing.text, &text);
  editing.text = text;
}

/// The byte just after the word of `caption` said over `at_ms` in the
/// source, in `text`, the caption opened for typing.
fn caret_after_word(caption: &Caption, text: &str, at_ms: f64) -> Option<usize> {
  let index = caption
    .lines
    .iter()
    .flatten()
    .position(|word| word.source_start_ms <= at_ms && at_ms < word.source_end_ms)?;
  screenwide_text::emphasis::words(text)
    .nth(index)
    .map(|word| word.end)
}
