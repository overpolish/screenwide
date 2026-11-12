// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation chrome shows and asks for: its grips, whether it owns
//! the screen, and the cursor over a point.

use super::picking::item_grips;
use super::*;
use screenwide_annotations::gesture::{MODE_HIGHLIGHT, MODE_TEXT};

/// The chosen arrow's grips, in device pixels, where they show through the
/// lenses over it, for the chrome to draw. Empty when no arrow is chosen or
/// the arrow tool has no say, which is what leaves the layer's own chrome
/// standing.
pub(crate) fn selected_grips(state: &SurfaceState, scale: f64) -> Vec<[f32; 2]> {
  if state.annotation.mode == MODE_NONE {
    return Vec::new();
  }
  let Ok(index) = usize::try_from(state.annotation.selected) else {
    return Vec::new();
  };
  let Some(item) = selected_item(state).copied() else {
    return Vec::new();
  };
  let Some(image) = image_frame(state) else {
    return Vec::new();
  };
  item_grips(image, &item)
    .into_iter()
    .map(|grip| {
      let ((x, y), _) = super::magnify_chrome::grip_shown(state, index, grip);
      [(x * scale) as f32, (y * scale) as f32]
    })
    .collect()
}

/// Whether the arrow chrome is what is on screen. A drawing tool always draws
/// its own chrome; the select tool only once it is holding an arrow or a
/// group, so an ordinary layer selection is untouched, while it draws a band,
/// or while it is held under the shortcut modifier.
pub(crate) fn owns_chrome(state: &SurfaceState) -> bool {
  drawing_kind(state.annotation.mode).is_some()
    || (state.annotation.mode != MODE_NONE
      && (state.annotation.selected >= 0
        || has_group(state)
        || state.annotation.marquee.is_some()
        || state.select_held))
}

/// The cursor the arrow tool asks for over `point`, or `None` when the
/// choice belongs to the layer underneath. The twin of `annotation_cursor`.
pub(crate) fn cursor_for(state: &SurfaceState, point: (f64, f64)) -> Option<CursorKind> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  // A band under way keeps the pointer whatever it passes over.
  if state
    .annotation
    .drag
    .is_some_and(|drag| drag.target_kind == TARGET_MARQUEE)
  {
    return Some(CursorKind::Arrow);
  }
  if super::typing::over_box(state, point) {
    return Some(CursorKind::IBeam);
  }
  // An annotation under the pointer is something to take hold of, so the
  // pointer says so; a box's grip resizes, and says which way. An image's
  // grips turn with it, and so do the ways they resize.
  let handle = handle_at_point(state, point);
  let resize = match (handle, selected_item(state)) {
    (Some(handle), Some(item)) if item.shape_kind() == AnnotationKind::Image => {
      super::image_chrome::grip_cursor(item, handle)
    }
    _ => handle,
  };
  if let Some(cursor) = resize.and_then(super::redact_chrome::grip_cursor) {
    return Some(cursor);
  }
  if handle.is_some() {
    return Some(CursorKind::Arrow);
  }
  // Past the chosen annotation's grips, the pen draws over whatever is under
  // it.
  if !picks_annotations(state.annotation.mode) {
    return Some(CursorKind::Crosshair);
  }
  if shaft_at_point(state, point).is_some() {
    return Some(CursorKind::Arrow);
  }
  // Empty picture: the select tool takes hold of a group inside its box, the
  // text tool takes typing and the highlight selects the text it covers, and
  // a drawing tool draws.
  if group_layer_at_point(state, point).is_some() {
    return Some(CursorKind::Arrow);
  }
  if matches!(state.annotation.mode, MODE_TEXT | MODE_HIGHLIGHT) {
    return Some(CursorKind::IBeam);
  }
  drawing_kind(state.annotation.mode)
    .is_some()
    .then_some(CursorKind::Crosshair)
}
