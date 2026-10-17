// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One text box being typed into.
//!
//! Every change is handed to the document as it is typed, inside one edit
//! gesture that React opens when the typing begins and closes when it ends,
//! so the typing lands in the history as one edit and the panel can dress the
//! box meanwhile. The document trails the typing by a round trip, so the text
//! typed so far is held here and laid over whatever list the document sends
//! back, with the words chosen in it carried along each change. The panel
//! may choose words or dress the box while it is typed into; what it chose
//! is taken up from the document's copy and carried on from there.

use super::model::{has_content, normalised_text};
use crate::{Annotation, AnnotationShape};
use screenwide_text::emphasis::follow_edit;
use screenwide_text::{TextSpan, TextStyle};
use serde::Serialize;
use ts_rs::TS;

/// Where a commit falls in the typing, which is what tells React to open and
/// close the edit gesture it groups the typing into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TextEditPhase {
  Begin,
  Update,
  End,
}

pub struct AnnotationTextEdit {
  id: String,
  text: String,
  /// The look the typing goes by, and the words chosen in the text.
  look: Box<TextStyle>,
  emphasis: Vec<TextSpan>,
  /// The newest change applied, so one delivered late is dropped.
  revision: u64,
}

impl AnnotationTextEdit {
  /// Opens a session on the text box `id`, holding `text` in `look` with
  /// the words of `emphasis` chosen.
  pub fn begin(id: String, text: String, look: &TextStyle, emphasis: Vec<TextSpan>) -> Self {
    Self {
      id,
      text,
      look: Box::new(look.clone()),
      emphasis,
      revision: 0,
    }
  }

  pub fn id(&self) -> &str {
    &self.id
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  /// Takes up the look and the words chosen in the document's copy of the
  /// box in `annotations`, where the panel changed either while the box is
  /// typed into. The copy may trail the typing, so its words are carried to
  /// the text typed since. Answers the text, the look and the words the
  /// typing goes by now, where the look or the words changed.
  pub fn adopt(&mut self, annotations: &[Annotation]) -> Option<(&str, &TextStyle, &[TextSpan])> {
    let AnnotationShape::Text {
      text,
      look,
      emphasis,
      ..
    } = &annotations
      .iter()
      .find(|annotation| annotation.id == self.id)?
      .shape
    else {
      return None;
    };
    let chosen = follow_edit(emphasis, text, &self.text);
    if chosen == self.emphasis && look == &self.look {
      return None;
    }
    self.emphasis = chosen;
    self.look.clone_from(look);
    Some((&self.text, &self.look, &self.emphasis))
  }

  /// Takes the text a change reported, unless a newer change already
  /// arrived, and carries the chosen words through it. Answers whether it
  /// was taken.
  pub fn take(&mut self, text: &str, revision: u64) -> bool {
    if revision <= self.revision {
      return false;
    }
    self.revision = revision;
    let text = normalised_text(text);
    self.emphasis = follow_edit(&self.emphasis, &self.text, &text);
    self.text = text;
    true
  }

  /// Lays the text typed so far, and the words chosen in it, over the box in
  /// `annotations`. Answers whether the box is there to take them.
  pub fn apply(&self, annotations: &mut [Annotation]) -> bool {
    let Some(AnnotationShape::Text { text, emphasis, .. }) = annotations
      .iter_mut()
      .find(|annotation| annotation.id == self.id)
      .map(|annotation| &mut annotation.shape)
    else {
      return false;
    };
    text.clone_from(&self.text);
    emphasis.clone_from(&self.emphasis);
    true
  }

  /// Closes the session. A box left with nothing readable in it is removed.
  /// Answers whether the box is still there.
  pub fn finish(self, annotations: &mut Vec<Annotation>) -> bool {
    self.apply(annotations);
    let empty = !has_content(&self.text);
    annotations.retain(|annotation| !(empty && annotation.id == self.id));
    !empty
      && annotations
        .iter()
        .any(|annotation| annotation.id == self.id)
  }
}
