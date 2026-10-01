// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The presses around a box being typed into: the double-click that opens
//! it, a press in it that places the caret or takes a selection, and a press
//! anywhere else that ends the typing and usually carries on.

use super::*;
use crate::editor::annotations::text::model::has_content;

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
/// this box or another, grab the OSC, choose a layer or pan. Two only end
/// it: one on empty picture under the text tool, since clicking away is how
/// typing is finished and must not drop a fresh box as well; and one that
/// leaves the box with nothing to read, which Rust removes, so the grips the
/// press would be read against may still count it. Answers whether typing
/// took the press.
pub(crate) fn press(inner: &Arc<SurfaceInner>, point: (f64, f64), double: bool) -> bool {
  let (in_box, carries_on) = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some((flat, readable)) = state
      .annotation
      .typing
      .as_ref()
      .map(|typing| (typing.flat, has_content(typing.edit.text())))
    else {
      return false;
    };
    let place = place(&state, flat).filter(|_| inside(&state, flat, point));
    if let (Some(place), Some(typing)) = (place.as_ref(), state.annotation.typing.as_mut()) {
      let offset = place.layout(|layout| typing.edit.offset_at(place.local(point), layout));
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
    let carries_on = !in_box && readable && !drops_text(&state, point);
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
  let Some(flat) = state
    .annotation
    .typing
    .as_ref()
    .filter(|typing| typing.selecting)
    .map(|typing| typing.flat)
  else {
    return true;
  };
  let place = place(&state, flat);
  if let (Some(place), Some(typing)) = (place, state.annotation.typing.as_mut()) {
    let before = typing.edit.marks(true);
    let offset = place.layout(|layout| typing.edit.offset_at(place.local(point), layout));
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
