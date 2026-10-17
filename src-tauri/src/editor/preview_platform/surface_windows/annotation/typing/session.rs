// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rust's side of a box being typed into: opening it, handing it the layout
//! of its words again when the panel chooses them, and taking it away.

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn set_annotation_text_callback(&mut self, callback: AnnotationTextCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.annotation_text = Some(callback);
    }
  }

  /// Opens the box at `index` in the published grips for typing, holding
  /// `text` set in `look`, with the words of `emphasis` chosen. The caret is
  /// type, which the shader inks the way it inks the text.
  pub(crate) fn begin_annotation_text(
    &self,
    index: usize,
    text: &str,
    look: &screenwide_text::TextStyle,
    emphasis: &[screenwide_text::TextSpan],
  ) {
    {
      let Ok(mut state) = self.inner.state.lock() else {
        return;
      };
      let opening = state.annotation.opening.take();
      let Some((layer, item)) = identity(&state, index) else {
        return;
      };
      let face = screenwide_annotations::text::look::face(look);
      let words = TypedWords::new(text, look, emphasis);
      let mut edit = TextTyping::new(text);
      // A box opened by a double-click puts the caret where the click
      // landed; one fresh from its own press has nothing typed to place it in.
      if let (Some(point), Some(place)) = (opening, place(&state, Target::Box(index), &face)) {
        let offset = place.layout(words.as_ref(), |lines| {
          edit.offset_at(place.local(point), lines)
        });
        edit.place(offset, false);
      }
      state.annotation.typing = Some(Typing {
        target: Target::Box(index),
        layer,
        index: item,
        face,
        words,
        edit,
        revision: 0,
        caret_on: true,
        selecting: false,
        surrogate: None,
        opening: None,
      });
      redraw_workspace(&self.inner, &mut state);
    }
    self.inner.editor.begin_typing();
  }

  /// Opens the caption showing for typing, holding `text` in `look`, the
  /// caption's style as it is typed into, with its key words `emphasis`;
  /// the caret at byte `caret` of `text`, or where the press that opened it
  /// landed once its words are laid out. Its words are always laid out by
  /// Rust, as the caption draws them.
  pub(crate) fn begin_caption_text(
    &self,
    text: &str,
    look: &screenwide_text::TextStyle,
    emphasis: &[screenwide_text::TextSpan],
    caret: Option<usize>,
  ) {
    {
      let Ok(mut state) = self.inner.state.lock() else {
        return;
      };
      let opening = state.annotation.opening.take();
      let mut caption = state.annotation.caption.unwrap_or_default();
      caption.layer_id = CAPTIONS_LAYER_ID;
      let mut edit = TextTyping::new(text);
      if let Some(caret) = caret {
        edit.place(caret, false);
      }
      state.annotation.typing = Some(Typing {
        target: Target::Caption(caption),
        layer: CAPTIONS_LAYER_ID,
        index: 0,
        face: screenwide_annotations::text::look::face(look),
        words: Some(TypedWords::caption(text, look, emphasis)),
        edit,
        revision: 0,
        caret_on: true,
        selecting: false,
        surrogate: None,
        opening: opening.filter(|_| caret.is_none()),
      });
      redraw_workspace(&self.inner, &mut state);
    }
    self.inner.editor.begin_typing();
  }

  /// Hands the box or `caption` being typed into the layout of its words
  /// again, `text` in `look` with the words of `emphasis` chosen, after the
  /// panel chose words or dressed it: what a press and a line move go by
  /// from now. The layout follows any change typed since Rust made it.
  pub(crate) fn set_annotation_text_words(
    &self,
    text: &str,
    look: &screenwide_text::TextStyle,
    emphasis: &[screenwide_text::TextSpan],
    caption: bool,
  ) {
    let Ok(mut state) = self.inner.state.lock() else {
      return;
    };
    if let Some(typing) = state
      .annotation
      .typing
      .as_mut()
      .filter(|typing| typing.is_caption() == caption)
    {
      typing.face = screenwide_annotations::text::look::face(look);
      typing.words = if caption {
        Some(TypedWords::caption(text, look, emphasis))
      } else {
        TypedWords::new(text, look, emphasis)
      };
      if let Some(words) = typing.words.as_mut() {
        words.follow(typing.edit.text());
      }
    }
  }

  /// Types `text` over bytes `start` to `end` of the caption being typed
  /// into, as typing would: it undoes with the typing and reports as a
  /// change of it.
  pub(crate) fn replace_caption_text(&self, start: usize, end: usize, text: &str) {
    let change = {
      let Ok(mut state) = self.inner.state.lock() else {
        return;
      };
      let Some(typing) = state
        .annotation
        .typing
        .as_mut()
        .filter(|typing| typing.is_caption())
      else {
        return;
      };
      if end > typing.edit.text().len() {
        return;
      }
      typing.edit.place(start, false);
      typing.edit.place(end, true);
      if !typing.edit.paste(text) {
        return;
      }
      typing.revision += 1;
      typing.caret_on = true;
      if let Some(words) = typing.words.as_mut() {
        words.follow(typing.edit.text());
      }
      let change = (
        (typing.layer, typing.index),
        typing.edit.text().to_owned(),
        typing.revision,
      );
      sync_typing_marks(&mut state);
      change
    };
    self.inner.editor.restart_blink();
    report(
      &self.inner,
      AnnotationTextPhase::Change,
      change.0,
      change.1,
      change.2,
    );
  }

  /// Takes the typing away, as Rust's own end of the session asks.
  pub(crate) fn end_annotation_text(&self) {
    end(&self.inner);
  }
}
