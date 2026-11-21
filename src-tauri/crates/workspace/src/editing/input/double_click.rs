// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A double-click that no text, caption or picked object took. On a layer,
//! with Select or the crop in hand, it asks for that layer's crop, as
//! Figma, Keynote and Descript open an image's crop; on an annotation it
//! does nothing, so a double-click meant for one never crops the layer
//! under it; anywhere else it puts the view back at 100%, centred.

use super::*;
use screenwide_annotations::gesture::{MODE_NONE, MODE_SELECT};

enum Answer {
  Crop(u32),
  Nothing,
  ResetView,
}

fn answer(inner: &SurfaceInner, state: &SurfaceState, point: (f64, f64)) -> Answer {
  // A pen in hand keeps the double-click it always had: a second press
  // landing quickly is the platform's, not a crop.
  if !matches!(state.annotation.mode, MODE_NONE | MODE_SELECT) {
    return Answer::ResetView;
  }
  if state.annotation.mode == MODE_SELECT && annotation::shaft_at_point(state, point).is_some() {
    return Answer::Nothing;
  }
  match shared_selection_hit(inner, state, point) {
    Some((selection, _))
      if selection.layer_id != FRAME_LAYER_ID
        && !crate::is_overlay_layer(selection.layer_id)
        && selection.blank == 0 =>
    {
      Answer::Crop(selection.layer_id)
    }
    Some(_) => Answer::Nothing,
    None => Answer::ResetView,
  }
}

pub(super) fn double_click(inner: &Arc<SurfaceInner>, point: (f64, f64)) {
  let answer = match inner.state.lock() {
    Ok(state) => answer(inner, &state, point),
    Err(_) => return,
  };
  match answer {
    Answer::Crop(layer_id) => emit_press(inner, PointerPress::Crop { layer_id }),
    Answer::Nothing => {}
    Answer::ResetView => reset_view(inner),
  }
}

/// Puts the view back at 100%, centred in the whole viewport.
pub(in crate::editing) fn reset_view(inner: &Arc<SurfaceInner>) {
  if let Ok(mut state) = inner.state.lock() {
    state.workspace_transform = WorkspaceTransform::default();
    apply_workspace_transform(inner, &mut state, false);
  }
}
