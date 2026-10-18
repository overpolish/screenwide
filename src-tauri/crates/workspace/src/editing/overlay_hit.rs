// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the overlays drawn over the picture show in the current frame, for
//! the press that opens their tools: the shortcut strip and the caption.

use super::{display_selection, PreviewSelection, SurfaceInner, SurfaceState};
use crate::{CAPTIONS_LAYER_ID, KEYBOARD_LAYER_ID};
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
  let scene = inner.host.scene();
  let bounds = if selection.layer_id == KEYBOARD_LAYER_ID {
    scene.keyboard_bounds(selection.pane_index)?
  } else {
    scene.caption_bounds(selection.pane_index)?
  };
  selection.x = bounds[0];
  selection.y = bounds[1];
  selection.width = bounds[2];
  selection.height = bounds[3];
  display_selection(state, selection)
}
