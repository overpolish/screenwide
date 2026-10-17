// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into a text box, or into the caption showing.
//!
//! The twin of `recording_preview_surface_macos+annotation_text.m` and its
//! `_input.m`. macOS lays a native text view over the box; nothing can be laid
//! over a DirectComposition visual that way, so the typing window takes the
//! keyboard, [`TextTyping`] keeps the edits, and the compositor draws the
//! caret and the selection into the box's own type. Every change goes to
//! Rust, which re-presents the box with the new text, as it does on macOS.
//!
//! As everywhere in this module, reports go out with no surface state held:
//! the callback comes straight back in to publish what changed.

use std::sync::Arc;

use super::*;
use crate::editor::preview_platform::CAPTIONS_LAYER_ID;
use screenwide_annotations::text::geometry::text_distance;
use screenwide_annotations::text::typed::TypedWords;
use screenwide_text::engine::face::TypeFace;
use screenwide_text::engine::typing::TextTyping;

mod keys;
mod place;
mod presses;
mod session;
use place::{place, Place};
pub(crate) use presses::{
  caption_offset_at, caption_target_at, open, open_caption, pointer_move, press, up,
};

/// What is typed into: the box at its place in the published grips, or the
/// caption showing on the captions layer it was pressed on.
#[derive(Clone, Copy)]
enum Target {
  Box(usize),
  Caption(PreviewSelection),
}

pub(crate) struct Typing {
  target: Target,
  /// The layer the box belongs to and its place in that layer's own list:
  /// what a report names it by, and what finds the pane it is drawn in.
  layer: u32,
  index: u32,
  /// The face the box's look sets its type in.
  face: TypeFace,
  /// The box's words as drawn, where its look sets chosen ones apart: what
  /// a press and a line move go by then, rather than lines set in `face`.
  words: Option<TypedWords>,
  edit: TextTyping,
  revision: u64,
  caret_on: bool,
  /// A press in the box is taking the selection with it.
  selecting: bool,
  /// The first half of a character outside the Basic Multilingual Plane,
  /// which arrives as two units.
  surrogate: Option<u16>,
  /// Where the double-click that opened a caption landed: its words are laid
  /// out a present after it opens, so the caret is put there once they are.
  opening: Option<(f64, f64)>,
}

impl Typing {
  fn is_caption(&self) -> bool {
    matches!(self.target, Target::Caption(_))
  }
}

/// The layer a box belongs to and its place in that layer's list, the
/// identity a gesture reports too.
fn identity(state: &SurfaceState, flat: usize) -> Option<(u32, u32)> {
  let item = state.annotation.handles.get(flat)?;
  let layer = if item.layer_id >= 0 {
    item.layer_id as u32
  } else {
    state.selection.map_or(0, |selection| selection.layer_id)
  };
  Some((layer, item.index))
}

fn inside(state: &SurfaceState, typing: &Typing, point: (f64, f64)) -> bool {
  let flat = match typing.target {
    Target::Box(flat) => flat,
    Target::Caption(_) => {
      return place(state, typing.target, &typing.face).is_some_and(|place| place.contains(point))
    }
  };
  let Some(item) = state.annotation.handles.get(flat) else {
    return false;
  };
  item_image_frame(state, flat as i32).is_some_and(|image| {
    text_distance(
      [point.0 as f32, point.1 as f32],
      &text_geometry(image, item),
    ) <= 0.0
  })
}

/// Whether `point` is over the box being typed into, for the I-beam.
pub(super) fn over_box(state: &SurfaceState, point: (f64, f64)) -> bool {
  state
    .annotation
    .typing
    .as_ref()
    .is_some_and(|typing| inside(state, typing, point))
}

/// Whether the typing has nothing left to type into: the tool put down, or
/// the box gone from the published grips. A caption is typed into whatever
/// tool is in hand.
pub(super) fn lost_its_box(state: &SurfaceState) -> bool {
  state
    .annotation
    .typing
    .as_ref()
    .is_some_and(|typing| match typing.target {
      Target::Caption(_) => false,
      Target::Box(flat) => {
        state.annotation.mode == MODE_NONE
          || state
            .annotation
            .handles
            .get(flat)
            .is_none_or(|item| item.kind != AnnotationKind::Text.raw())
      }
    })
}

/// Hands every pane the caret and selection it draws: the pane the typed
/// box's layer is drawn in gets them, every other pane none, and the pane a
/// caption is typed into on gets them for its caption. A screenshot pane
/// knows its layer by token; a recording pane is its own layer. A caption's
/// caret waiting for its words to be laid out is put where the press that
/// opened it landed, once they are.
pub(in crate::editor::preview_platform::surface_windows) fn sync_typing_marks(
  state: &mut SurfaceState,
) {
  let opened = state.annotation.typing.as_ref().and_then(|typing| {
    let point = typing.opening?;
    let place = place(state, typing.target, &typing.face)?;
    Some(place.layout(typing.words.as_ref(), |lines| {
      typing.edit.offset_at(place.local(point), lines)
    }))
  });
  if let (Some(offset), Some(typing)) = (opened, state.annotation.typing.as_mut()) {
    typing.edit.place(offset, false);
    typing.opening = None;
  }
  let typing = state.annotation.typing.as_ref().map(|typing| {
    (
      typing.target,
      u64::from(typing.layer),
      typing.index as usize,
      typing.edit.marks(typing.caret_on),
    )
  });
  for (index, pane) in state
    .panes
    .iter_mut()
    .enumerate()
    .filter_map(|(index, pane)| pane.as_mut().map(|pane| (index, pane)))
  {
    pane.annotation_typing = None;
    pane.caption_typing = None;
    match typing {
      Some((Target::Caption(caption), _, _, marks)) if caption.pane_index as usize == index => {
        pane.caption_typing = Some(marks);
      }
      Some((Target::Box(_), layer, typed, marks))
        if pane.source_token == Some(layer) || layer == index as u64 =>
      {
        pane.annotation_typing = Some((typed, marks));
      }
      _ => {}
    }
  }
}

fn report(
  inner: &SurfaceInner,
  phase: AnnotationTextPhase,
  typing: (u32, u32),
  text: String,
  revision: u64,
) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.annotation_text.as_mut() {
      callback(phase, typing.0, typing.1, text, revision);
    }
  }
}

/// Takes the typing away and redraws the box without its caret. Nothing is
/// reported: Rust's own end asks for this, and a local end reports itself.
fn end(inner: &Arc<SurfaceInner>) -> Option<Typing> {
  let typing = {
    let mut state = inner.state.lock().ok()?;
    let typing = state.annotation.typing.take()?;
    redraw_workspace(inner, &mut state);
    typing
  };
  inner.editor.end_typing();
  Some(typing)
}

/// Ends the typing here - Escape, a press outside the box, the keyboard gone
/// elsewhere - and tells Rust the text it ended with.
fn finish(inner: &Arc<SurfaceInner>) {
  if let Some(typing) = end(inner) {
    let text = typing.edit.text().to_owned();
    report(
      inner,
      AnnotationTextPhase::End,
      (typing.layer, typing.index),
      text,
      typing.revision + 1,
    );
  }
}

/// What the typing window hands on: keys and typed text, the blink, and the
/// ways the typing ends.
pub(in crate::editor::preview_platform::surface_windows) fn handle_typing_input(
  editor: HWND,
  input: editor::TypingInput,
) {
  let Some(inner) = surface_for_editor(editor) else {
    return;
  };
  match input {
    editor::TypingInput::Released => inner.editor.focus_webview(),
    editor::TypingInput::Finish | editor::TypingInput::FocusLost => finish(&inner),
    editor::TypingInput::Blink => {
      if let Ok(mut state) = inner.state.lock() {
        if let Some(typing) = state.annotation.typing.as_mut() {
          typing.caret_on = !typing.caret_on;
          redraw_workspace(&inner, &mut state);
        }
      }
    }
    editor::TypingInput::Key { .. } | editor::TypingInput::Char(_) => keys::edit(&inner, input),
  }
}
