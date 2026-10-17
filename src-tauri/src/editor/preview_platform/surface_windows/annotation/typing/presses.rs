// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The presses around a box being typed into: the double-click that opens
//! it, a press in it that places the caret or takes a selection, and a press
//! anywhere else that ends the typing and usually carries on.

use super::*;
use screenwide_annotations::text::model::has_content;

/// A double-click on a text box with the annotation tools in hand opens it
/// for typing, with the caret where it landed. Answers whether it did.
pub(crate) fn open(inner: &Arc<SurfaceInner>, point: (f64, f64)) -> bool {
  let identity = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    if state.annotation.typing.is_some() || state.annotation.mode == MODE_NONE {
      return false;
    }
    let Some(shaft) = shaft_at_point(&state, point)
      .filter(|shaft| state.annotation.handles[*shaft].kind == AnnotationKind::Text.raw())
    else {
      return false;
    };
    // The first click chose the box and may have armed a move; the second
    // opens it for typing instead.
    state.annotation.drag = None;
    state.annotation.press_taken = true;
    state.annotation.opening = Some(point);
    identity(&state, shaft)
  };
  if let Some(identity) = identity {
    report(inner, AnnotationTextPhase::Open, identity, String::new(), 0);
  }
  true
}

/// The caption showing under `point`, as the captions layer a press on it
/// reports, wherever no drawing tool is in hand. It is found where its pane
/// draws it rather than among the layers a press picks: the select tool's
/// first click on the caption brings up the Captions tool, which picks no
/// layers, so a double-click's second press lands there.
pub(crate) fn caption_target_at(
  inner: &SurfaceInner,
  state: &SurfaceState,
  point: (f64, f64),
) -> Option<PreviewSelection> {
  if state.annotation.mode != MODE_NONE {
    return None;
  }
  // The twin of the captions target React hands the select tool: the whole
  // canvas of the screen's pane, tested against where the caption shows.
  let caption = PreviewSelection {
    pane_index: 0,
    layer_id: CAPTIONS_LAYER_ID,
    radius_disabled: 1,
    width: 1.0,
    height: 1.0,
    ..PreviewSelection::default()
  };
  overlay_hit::frame(inner, state, caption)
    .filter(|frame| {
      (frame.x..=frame.x + frame.width).contains(&point.0)
        && (frame.y..=frame.y + frame.height).contains(&point.1)
    })
    .map(|_| caption)
}

/// A double-click on the caption showing, with the select tool or the
/// Captions tool in hand, chooses the captions and opens the caption for
/// typing, with the caret
/// where it landed once its words are laid out. `point` is the press in the
/// viewport, `on_layer` the same press carried onto the layer standing
/// upright, where the typing reads presses. Answers whether it did.
pub(crate) fn open_caption(
  inner: &Arc<SurfaceInner>,
  point: (f64, f64),
  on_layer: (f64, f64),
) -> bool {
  {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    if state.annotation.typing.is_some() {
      return false;
    }
    let Some(target) = caption_target_at(inner, &state, point) else {
      return false;
    };
    state.annotation.drag = None;
    state.annotation.press_taken = true;
    state.annotation.opening = Some(on_layer);
    state.annotation.caption = Some(target);
  }
  emit_selection(inner, Some(CAPTIONS_LAYER_ID));
  report(
    inner,
    AnnotationTextPhase::Open,
    (CAPTIONS_LAYER_ID, 0),
    String::new(),
    0,
  );
  true
}

/// The byte of the caption being typed into's text a right press at
/// `point`, carried onto the layer standing upright, lands at. None where no
/// caption is typed into or the press is off it.
pub(crate) fn caption_offset_at(inner: &SurfaceInner, point: (f64, f64)) -> Option<usize> {
  let state = inner.state.lock().ok()?;
  let typing = state
    .annotation
    .typing
    .as_ref()
    .filter(|typing| typing.is_caption())?;
  let place = place(&state, typing.target, &typing.face).filter(|place| place.contains(point))?;
  Some(place.layout(typing.words.as_ref(), |lines| {
    typing.edit.offset_at(place.local(point), lines)
  }))
}

/// Whether a press at `point` would drop a fresh text box: empty picture
/// under the text tool, exactly as `gesture::down` reads it.
fn drops_text(state: &SurfaceState, point: (f64, f64)) -> bool {
  drawing_kind(state.annotation.mode) == Some(AnnotationKind::Text)
    && handle_at_point(state, point).is_none()
    && shaft_at_point(state, point).is_none()
}

/// A press while a box is typed into. In the box it places the caret, and a
/// double-click selects the word it lands on. Anywhere else it ends the
/// typing and then carries on as the press it is, so one press can pick up
/// this box or another, grab the OSC, choose a layer or pan - or, on empty
/// picture under the text tool, drop the next box, so a run of boxes takes a
/// press each. Only one press just ends it: one that leaves the box with
/// nothing to read and lands on something other than empty picture, since
/// Rust removes that box, so the grips the press would be read against may
/// still count it; a fresh box is read against no grip. Answers whether
/// typing took the press.
pub(crate) fn press(inner: &Arc<SurfaceInner>, point: (f64, f64), double: bool) -> bool {
  let (in_box, carries_on) = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some((target, readable, face, caption)) = state.annotation.typing.as_ref().map(|typing| {
      (
        typing.target,
        has_content(typing.edit.text()),
        typing.face.clone(),
        typing.is_caption(),
      )
    }) else {
      return false;
    };
    let place = place(&state, target, &face).filter(|_| {
      state
        .annotation
        .typing
        .as_ref()
        .is_some_and(|typing| inside(&state, typing, point))
    });
    if let (Some(place), Some(typing)) = (place.as_ref(), state.annotation.typing.as_mut()) {
      let offset = place.layout(typing.words.as_ref(), |lines| {
        typing.edit.offset_at(place.local(point), lines)
      });
      if double {
        typing.edit.select_word(offset);
      } else {
        typing.edit.place(offset, keys::shift_held());
      }
      typing.selecting = !double;
      typing.caret_on = true;
      redraw_composed_panes(inner, &mut state);
    }
    let in_box = place.is_some();
    // A caption left with nothing to read only hides its words, so a press
    // anywhere else always carries on.
    let carries_on = !in_box && (caption || readable || drops_text(&state, point));
    state.annotation.press_taken = !carries_on;
    (in_box, carries_on)
  };
  if in_box {
    inner.editor.restart_blink();
  } else {
    finish(inner);
  }
  !carries_on
}

/// The drag after a press typing took: in the box it takes the selection
/// with it, anywhere else it belongs to nothing.
pub(crate) fn pointer_move(inner: &Arc<SurfaceInner>, point: (f64, f64)) -> bool {
  let Ok(mut state) = inner.state.lock() else {
    return false;
  };
  if !state.annotation.press_taken {
    return false;
  }
  let Some((target, face)) = state
    .annotation
    .typing
    .as_ref()
    .filter(|typing| typing.selecting)
    .map(|typing| (typing.target, typing.face.clone()))
  else {
    return true;
  };
  let place = place(&state, target, &face);
  if let (Some(place), Some(typing)) = (place, state.annotation.typing.as_mut()) {
    let before = typing.edit.marks(true);
    let offset = place.layout(typing.words.as_ref(), |lines| {
      typing.edit.offset_at(place.local(point), lines)
    });
    typing.edit.place(offset, true);
    if typing.edit.marks(true) != before {
      redraw_composed_panes(inner, &mut state);
    }
  }
  true
}

/// The release after a press typing took, which belongs to nothing else.
pub(crate) fn up(inner: &Arc<SurfaceInner>) -> bool {
  let Ok(mut state) = inner.state.lock() else {
    return false;
  };
  let taken = std::mem::take(&mut state.annotation.press_taken);
  if let Some(typing) = state.annotation.typing.as_mut() {
    typing.selecting = false;
  }
  taken
}
