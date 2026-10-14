// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the overlays drawn over the picture show in the current frame, for
//! the press that opens their tools: the shortcut strip and the caption.

use super::{display_selection, PreviewSelection, SurfaceInner, SurfaceState};
use crate::editor::preview_platform::{CAPTIONS_LAYER_ID, KEYBOARD_LAYER_ID};
use screenwide_compositor::annotation_gpu::caption_visible_bounds;
use screenwide_compositor::PreviewSurfaceRect;

/// The frame a press on `selection` is tested against: a layer's own, or for
/// an overlay where it is drawn now, and none while it shows nothing.
pub(super) fn frame(
  inner: &SurfaceInner,
  state: &SurfaceState,
  mut selection: PreviewSelection,
) -> Option<PreviewSurfaceRect> {
  if selection.layer_id != KEYBOARD_LAYER_ID && selection.layer_id != CAPTIONS_LAYER_ID {
    return display_selection(state, selection);
  }
  let pane = state.panes.get(selection.pane_index as usize)?.as_ref()?;
  let settings = pane.settings.as_ref()?;
  let canvas = (settings.width, settings.height);
  let bounds = if selection.layer_id == KEYBOARD_LAYER_ID {
    let overlay = pane.last_composition?.keyboard?;
    inner
      .gpu
      .compositor
      .keyboard_visible_bounds(&overlay, canvas)
      .ok()??
  } else {
    let (track, moment) = pane.captions.as_ref()?;
    caption_visible_bounds(track, *moment, canvas)?
  };
  selection.x = bounds[0];
  selection.y = bounds[1];
  selection.width = bounds[2];
  selection.height = bounds[3];
  display_selection(state, selection)
}
