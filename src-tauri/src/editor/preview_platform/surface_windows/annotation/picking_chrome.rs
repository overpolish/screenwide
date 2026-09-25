// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the annotation chrome shows and asks for: its grips, whether it owns
//! the screen, and the cursor over a point. The twin of
//! `recording_preview_surface_macos+annotation_chrome.m`.

use super::picking::item_grips;
use super::*;
use crate::editor::annotations::gesture::MODE_TEXT;

/// The chosen arrow's three grips, in device pixels, for the chrome to draw.
/// Empty when no arrow is chosen or the arrow tool has no say, which is what
/// leaves the layer's own chrome standing.
pub(crate) fn selected_grips(state: &SurfaceState, scale: f64) -> Vec<[f32; 2]> {
  if state.annotation.mode == MODE_NONE {
    return Vec::new();
  }
  let Some(item) = selected_item(state).copied() else {
    return Vec::new();
  };
  let Some(image) = image_frame(state) else {
    return Vec::new();
  };
  item_grips(image, &item)
    .into_iter()
    .map(|(x, y)| [(x * scale) as f32, (y * scale) as f32])
    .collect()
}

/// Whether the arrow chrome is what is on screen. The arrow tool always
/// draws its own chrome; the select tool only once it is holding an arrow, so
/// an ordinary layer selection is untouched. The twin of
/// `annotation_owns_chrome`.
pub(crate) fn owns_chrome(state: &SurfaceState) -> bool {
  drawing_kind(state.annotation.mode).is_some()
    || (state.annotation.mode != MODE_NONE && state.annotation.selected >= 0)
}

/// The cursor the arrow tool asks for over `point`, or `None` when the
/// choice belongs to the layer underneath. The twin of `annotation_cursor`.
pub(crate) fn cursor_for(state: &SurfaceState, point: (f64, f64)) -> Option<editor::CursorKind> {
  if state.annotation.mode == MODE_NONE {
    return None;
  }
  if super::typing::over_box(state, point) {
    return Some(editor::CursorKind::IBeam);
  }
  // An annotation under the pointer is something to take hold of, so the
  // pointer says so; a redaction's grip resizes, and says which way.
  let handle = handle_at_point(state, point);
  if let Some(cursor) = handle.and_then(super::redact_chrome::grip_cursor) {
    return Some(cursor);
  }
  if handle.is_some() || shaft_at_point(state, point).is_some() {
    return Some(editor::CursorKind::Arrow);
  }
  // Empty picture: the text tool takes typing, a drawing tool draws.
  if state.annotation.mode == MODE_TEXT {
    return Some(editor::CursorKind::IBeam);
  }
  drawing_kind(state.annotation.mode)
    .is_some()
    .then_some(editor::CursorKind::Crosshair)
}
