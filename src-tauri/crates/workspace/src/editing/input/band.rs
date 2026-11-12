// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A Select press that would have drawn a band but was let go where it went
//! down: the click it is to the layers.

use super::*;
use annotation::BandClick;

/// Lets go of a band that never travelled, making the layer press it stood
/// in for. Answers whether there was one. Either way the click is reported as
/// one change of choice, which the editor applies whole: a layer chosen lets
/// the annotations go with it, and nothing chosen lets go of both, so the
/// editor never draws the layer chosen without its annotations in between.
pub(super) fn released(inner: &Arc<SurfaceInner>, point: (f64, f64)) -> bool {
  let chose = {
    let Ok(mut state) = inner.state.lock() else {
      return false;
    };
    let Some(click) = annotation::band_click(&mut state) else {
      return false;
    };
    state.last_pointer = point;
    state.annotation.selected = -1;
    clear_selection_snap_guides(&mut state);
    let chose = match click {
      // The frame's own selection belongs to the Frame tool, which a click
      // never puts down.
      BandClick::ChoosesNothing => {
        if state
          .selection
          .is_some_and(|selection| selection.layer_id == FRAME_LAYER_ID)
        {
          None
        } else {
          state.selection = None;
          Some(None)
        }
      }
      BandClick::Chooses(selection) => {
        state.selection = Some(selection);
        Some(Some(selection.layer_id))
      }
    };
    draw_selection(inner, &state);
    chose
  };
  if let Some(selection) = chose {
    emit_selection(inner, selection);
  }
  refresh_cursor_for(inner);
  true
}
