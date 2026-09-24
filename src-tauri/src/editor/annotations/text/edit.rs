// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One text box being typed into.
//!
//! Every change is handed to the document as it is typed, inside one edit
//! gesture that React opens when the typing begins and closes when it ends,
//! so the typing lands in the history as one edit and the panel can dress the
//! box meanwhile. The document trails the typing by a round trip, so the text
//! typed so far is held here and laid over whatever list the document sends
//! back.

use super::model::{has_content, normalised_text};
use crate::editor::annotations::{Annotation, AnnotationShape};
use serde::Serialize;

/// Where a commit falls in the typing, which is what tells React to open and
/// close the edit gesture it groups the typing into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TextEditPhase {
  Begin,
  Update,
  End,
}

pub(crate) struct AnnotationTextEdit {
  id: String,
  text: String,
  /// The newest change applied, so one delivered late is dropped.
  revision: u64,
}

impl AnnotationTextEdit {
  /// Opens a session on the text box `id`, holding `text`.
  pub(crate) fn begin(id: String, text: String) -> Self {
    Self {
      id,
      text,
      revision: 0,
    }
  }

  pub(crate) fn id(&self) -> &str {
    &self.id
  }

  /// Takes the text a change reported, unless a newer change already
  /// arrived. Answers whether it was taken.
  pub(crate) fn take(&mut self, text: &str, revision: u64) -> bool {
    if revision <= self.revision {
      return false;
    }
    self.revision = revision;
    self.text = normalised_text(text);
    true
  }

  /// Lays the text typed so far over the box in `annotations`. Answers
  /// whether the box is there to take it.
  pub(crate) fn apply(&self, annotations: &mut [Annotation]) -> bool {
    let Some(AnnotationShape::Text { text, .. }) = annotations
      .iter_mut()
      .find(|annotation| annotation.id == self.id)
      .map(|annotation| &mut annotation.shape)
    else {
      return false;
    };
    text.clone_from(&self.text);
    true
  }

  /// Closes the session. A box left with nothing readable in it is removed.
  /// Answers whether the box is still there.
  pub(crate) fn finish(self, annotations: &mut Vec<Annotation>) -> bool {
    self.apply(annotations);
    let empty = !has_content(&self.text);
    annotations.retain(|annotation| !(empty && annotation.id == self.id));
    !empty
      && annotations
        .iter()
        .any(|annotation| annotation.id == self.id)
  }
}
