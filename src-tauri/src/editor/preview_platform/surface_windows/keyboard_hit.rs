// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Current-frame keyboard bounds used for body hit-testing.

use super::{display_selection, PreviewSelection, PreviewSurfaceRect, SurfaceInner, SurfaceState};

pub(super) fn frame(
  inner: &SurfaceInner,
  state: &SurfaceState,
  mut selection: PreviewSelection,
) -> Option<PreviewSurfaceRect> {
  if selection.layer_id != u32::MAX - 1 {
    return display_selection(state, selection);
  }
  let pane = state.panes.get(selection.pane_index as usize)?.as_ref()?;
  let settings = pane.settings.as_ref()?;
  let overlay = pane.last_composition?.keyboard?;
  let bounds = inner
    .gpu
    .compositor
    .keyboard_visible_bounds(
      &inner.gpu.device,
      &overlay,
      (settings.width, settings.height),
    )
    .ok()??;
  selection.x = bounds[0];
  selection.y = bounds[1];
  selection.width = bounds[2];
  selection.height = bounds[3];
  display_selection(state, selection)
}
