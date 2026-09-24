// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing into a text box.
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
use crate::editor::annotations::text::geometry::text_distance;
use crate::editor::annotations::text::typing::{TextTyping, TypingLayout};
use crate::editor::preview_platform::surface::type_device::line_width;

#[path = "typing_keys.rs"]
mod keys;
#[path = "typing_presses.rs"]
mod presses;
pub(crate) use presses::{open, pointer_move, press, up};

pub(crate) struct Typing {
  /// The box's place in the published grips.
  flat: usize,
  /// The layer the box belongs to and its place in that layer's own list:
  /// what a report names it by, and what finds the pane it is drawn in.
  layer: u32,
  index: u32,
  edit: TextTyping,
  revision: u64,
  caret_on: bool,
  /// A press in the box is taking the selection with it.
  selecting: bool,
  /// The first half of a character outside the Basic Multilingual Plane,
  /// which arrives as two units.
  surrogate: Option<u16>,
}

/// Where a box sets its lines on screen, in display points.
struct Place {
  left: f64,
  top: f64,
  width: f64,
  font: f64,
  align: u32,
}

impl Place {
  fn layout<R>(&self, work: impl FnOnce(&TypingLayout<'_>) -> R) -> R {
    let width = |line: &str| line_width(line, self.font);
    work(&TypingLayout {
      width: &width,
      block_width: self.width,
      font: self.font,
      align: self.align,
    })
  }

  fn local(&self, point: (f64, f64)) -> (f64, f64) {
    (point.0 - self.left, point.1 - self.top)
  }
}

fn place(state: &SurfaceState, flat: usize) -> Option<Place> {
  let item = state
    .annotation
    .handles
    .get(flat)
    .filter(|item| item.kind == AnnotationKind::Text.raw())?;
  let image = item_image_frame(state, flat as i32)?;
  let centre = text_geometry(image, item).start_head.c;
  let (width, height) = (item.middle_x * image.width, item.middle_y * image.width);
  Some(Place {
    left: f64::from(centre[0]) - width / 2.0,
    top: f64::from(centre[1]) - height / 2.0,
    width,
    font: item.width * image.width,
    align: item.start_head as u32 & 3,
  })
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

fn inside(state: &SurfaceState, flat: usize, point: (f64, f64)) -> bool {
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
    .is_some_and(|typing| inside(state, typing.flat, point))
}

/// Whether the typing has nothing left to type into: the tool put down, or
/// the box gone from the published grips.
pub(super) fn lost_its_box(state: &SurfaceState) -> bool {
  state.annotation.typing.as_ref().is_some_and(|typing| {
    state.annotation.mode == MODE_NONE
      || state
        .annotation
        .handles
        .get(typing.flat)
        .is_none_or(|item| item.kind != AnnotationKind::Text.raw())
  })
}

/// Hands every pane the caret and selection it draws: the pane the typed
/// box's layer is drawn in gets them, every other pane none. A screenshot
/// pane knows its layer by token; a recording pane is its own layer.
pub(in crate::editor::preview_platform::surface) fn sync_typing_marks(state: &mut SurfaceState) {
  let typing = state.annotation.typing.as_ref().map(|typing| {
    (
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
    pane.annotation_typing = typing
      .filter(|(layer, _, _)| pane.source_token == Some(*layer) || *layer == index as u64)
      .map(|(_, typed, marks)| (typed, marks));
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
    redraw_composed_panes(inner, &mut state);
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

impl RecordingPreviewSurface {
  pub(crate) fn set_annotation_text_callback(&mut self, callback: AnnotationTextCallback) {
    if let Ok(mut callbacks) = self.inner.callbacks.lock() {
      callbacks.annotation_text = Some(callback);
    }
  }

  /// Opens the box at `index` in the published grips for typing, holding
  /// `text`. macOS colours its caret by `_dark_ink`; here the caret is type,
  /// which the shader inks the way it inks the text.
  pub(crate) fn begin_annotation_text(&self, index: usize, text: &str, _dark_ink: bool) {
    {
      let Ok(mut state) = self.inner.state.lock() else {
        return;
      };
      let opening = state.annotation.opening.take();
      let Some((layer, item)) = identity(&state, index) else {
        return;
      };
      let mut edit = TextTyping::new(text);
      // A box opened by a double-click puts the caret where the click
      // landed; one fresh from its own press has nothing typed to place it in.
      if let (Some(point), Some(place)) = (opening, place(&state, index)) {
        let offset = place.layout(|layout| edit.offset_at(place.local(point), layout));
        edit.place(offset, false);
      }
      state.annotation.typing = Some(Typing {
        flat: index,
        layer,
        index: item,
        edit,
        revision: 0,
        caret_on: true,
        selecting: false,
        surrogate: None,
      });
      redraw_composed_panes(&self.inner, &mut state);
    }
    self.inner.editor.begin_typing();
  }

  /// Takes the typing away, as Rust's own end of the session asks.
  pub(crate) fn end_annotation_text(&self) {
    end(&self.inner);
  }
}

/// What the typing window hands on: keys and typed text, the blink, and the
/// ways the typing ends.
pub(in crate::editor::preview_platform::surface) fn handle_typing_input(
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
          redraw_composed_panes(&inner, &mut state);
        }
      }
    }
    editor::TypingInput::Key { .. } | editor::TypingInput::Char(_) => keys::edit(&inner, input),
  }
}
